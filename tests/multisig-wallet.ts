import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MultisigWallet } from "../target/types/multisig_wallet";
import { assert } from "chai";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

describe("multisig-wallet", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.multisigWallet as Program<MultisigWallet>;

  // Multisig state setup
  const owner1 = Keypair.generate();
  const owner2 = Keypair.generate();
  const owner3 = Keypair.generate();
  const owners = [owner1.publicKey, owner2.publicKey, owner3.publicKey];
  const threshold = new anchor.BN(2);

  // PDA derivation
  const createKey = Keypair.generate().publicKey;
  const [multisigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("multisig"), createKey.toBuffer()],
    program.programId
  );
  const [multisigSignerPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("multisig_signer"), multisigPda.toBuffer()],
    program.programId
  );

  // Transaction constraints
  const transactionAccount = Keypair.generate();
  const receiver = Keypair.generate();
  const transferAmount = 1 * anchor.web3.LAMPORTS_PER_SOL;

  before(async () => {
    // Airdrop SOL for test execution
    const signatures = await Promise.all([
      provider.connection.requestAirdrop(owner1.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(owner2.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(provider.wallet.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(multisigSignerPda, 5 * anchor.web3.LAMPORTS_PER_SOL) // Fund the multisig signer PDA
    ]);
    
    // Confirm airdrops
    for (const sig of signatures) {
        const latestBlockHash = await provider.connection.getLatestBlockhash();
        await provider.connection.confirmTransaction({
            blockhash: latestBlockHash.blockhash,
            lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
            signature: sig,
        });
    }
  });

  it("1. Creates a multisig wallet", async () => {
    await program.methods
      .createMultisig(createKey, owners, threshold)
      .accountsStrict({
        multisig: multisigPda,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const multisigState = await program.account.multisig.fetch(multisigPda);
    assert.strictEqual(multisigState.createKey.toBase58(), createKey.toBase58());
    assert.strictEqual(multisigState.threshold.toNumber(), 2);
    assert.strictEqual(multisigState.owners.length, 3);
    assert.strictEqual(multisigState.owners[0].toBase58(), owners[0].toBase58());
    assert.strictEqual(multisigState.nonce.toNumber(), 0);
  });

  it("2. Creates a transaction", async () => {
    // Dummy transaction: System program transfer from PDA to receiver
    const transferIx = SystemProgram.transfer({
      fromPubkey: multisigSignerPda,
      toPubkey: receiver.publicKey,
      lamports: transferAmount,
    });

    const keys = transferIx.keys.map(k => {
      return {
        pubkey: k.pubkey,
        isSigner: k.isSigner, 
        isWritable: k.isWritable
      };
    });

    await program.methods
      .createTransaction(transferIx.programId, keys, Buffer.from(transferIx.data))
      .accountsStrict({
        multisig: multisigPda,
        transaction: transactionAccount.publicKey,
        proposer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([transactionAccount])
      .rpc();

    const txState = await program.account.transaction.fetch(transactionAccount.publicKey);
    assert.strictEqual(txState.programId.toBase58(), SystemProgram.programId.toBase58());
    assert.strictEqual(txState.multisig.toBase58(), multisigPda.toBase58());
    assert.strictEqual(txState.executed, false);
    assert.strictEqual(txState.signers.length, 0);

    const multisigState = await program.account.multisig.fetch(multisigPda);
    assert.strictEqual(multisigState.nonce.toNumber(), 1);
  });

  it("3. Approves a transaction", async () => {
    // Owner 1 approves
    await program.methods
      .approve()
      .accountsStrict({
        multisig: multisigPda,
        transaction: transactionAccount.publicKey,
        owner: owner1.publicKey,
      })
      .signers([owner1])
      .rpc();

    let txState = await program.account.transaction.fetch(transactionAccount.publicKey);
    assert.strictEqual(txState.signers.length, 1);
    assert.strictEqual(txState.signers[0].toBase58(), owner1.publicKey.toBase58());

    // Owner 2 approves
    await program.methods
      .approve()
      .accountsStrict({
        multisig: multisigPda,
        transaction: transactionAccount.publicKey,
        owner: owner2.publicKey,
      })
      .signers([owner2])
      .rpc();

    txState = await program.account.transaction.fetch(transactionAccount.publicKey);
    assert.strictEqual(txState.signers.length, 2); // Threshold reached
  });

  it("4. Executes a transaction", async () => {
    const receiverBalanceBefore = await provider.connection.getBalance(receiver.publicKey);
    assert.strictEqual(receiverBalanceBefore, 0);

    // Provide the required accounts for the invoked inner instruction.
    // For SystemProgram.transfer, we need the from, to, and the system program.
    await program.methods
      .executeTransaction()
      .accountsStrict({
        multisig: multisigPda,
        multisigSigner: multisigSignerPda,
        transaction: transactionAccount.publicKey,
      })
      .remainingAccounts([
        { pubkey: multisigSignerPda, isWritable: true, isSigner: false },
        { pubkey: receiver.publicKey, isWritable: true, isSigner: false },
        { pubkey: SystemProgram.programId, isWritable: false, isSigner: false }
      ])
      .rpc();

    const txState = await program.account.transaction.fetch(transactionAccount.publicKey);
    assert.strictEqual(txState.executed, true);

    const receiverBalanceAfter = await provider.connection.getBalance(receiver.publicKey);
    assert.strictEqual(receiverBalanceAfter, transferAmount);
  });
});
