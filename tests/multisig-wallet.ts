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
  const createKey = Keypair.generate();
  const owner1 = Keypair.generate();
  const owner2 = Keypair.generate();
  const owner3 = Keypair.generate();
  
  // Members
  const members = [
    { key: owner1.publicKey, permission: { mask: 7 } },
    { key: owner2.publicKey, permission: { mask: 7 } },
    { key: owner3.publicKey, permission: { mask: 7 } },
  ];
  // Sort members by key as done in the program
  members.sort((a, b) => a.key.toBuffer().compare(b.key.toBuffer()));
  
  // PDAs
  const [multisigPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("multisig"), Buffer.from("multisig"), createKey.publicKey.toBuffer()],
    program.programId
  );
  
  const transactionIndex = new anchor.BN(1);
  const [proposalPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("multisig"), 
      multisigPda.toBuffer(), 
      Buffer.from("transaction"), 
      transactionIndex.toArrayLike(Buffer, "le", 8), 
      Buffer.from("proposal")
    ],
    program.programId
  );

  before(async () => {
    // Airdrop SOL for test execution
    const signatures = await Promise.all([
      provider.connection.requestAirdrop(createKey.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(owner1.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(owner2.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(owner3.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
      provider.connection.requestAirdrop(provider.wallet.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL),
    ]);
    
    for (const sig of signatures) {
        const latestBlockHash = await provider.connection.getLatestBlockhash();
        await provider.connection.confirmTransaction({
            blockhash: latestBlockHash.blockhash,
            lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
            signature: sig,
        });
    }
  });

  it("1. Creates a multisig", async () => {
    await program.methods
      .multisigCreate({
        configAuthority: null,
        threshold: 2,
        timeLock: 0,
        members: members,
        memo: new anchor.BN(0)
      })
      .accountsStrict({
        multisig: multisigPda,
        createKey: createKey.publicKey,
        creator: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([createKey])
      .rpc();

    const multisigState = await program.account.multisig.fetch(multisigPda);
    assert.strictEqual(multisigState.createKey.toBase58(), createKey.publicKey.toBase58());
    assert.strictEqual(multisigState.threshold, 2);
    assert.strictEqual(multisigState.members.length, 3);
  });

  it("2. Creates a proposal", async () => {
    await program.methods
      .proposalCreate({
        transactionIndex: transactionIndex,
        draft: false,
      })
      .accountsStrict({
        multisig: multisigPda,
        proposal: proposalPda,
        rentPayer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const proposalState = await program.account.proposal.fetch(proposalPda);
    assert.strictEqual(proposalState.multisig.toBase58(), createKey.publicKey.toBase58());
    assert.isTrue(proposalState.transactionIndex.eq(transactionIndex));
    assert.ok(proposalState.status.active);
  });

  it("3. Approves a proposal", async () => {
    await program.methods
      .approveProposal({
        memo: null,
      })
      .accountsStrict({
        multisig: multisigPda,
        member: owner1.publicKey,
        proposal: proposalPda,
      })
      .signers([owner1])
      .rpc();

    let proposalState = await program.account.proposal.fetch(proposalPda);
    assert.strictEqual(proposalState.approved.length, 1);
    assert.strictEqual(proposalState.approved[0].toBase58(), owner1.publicKey.toBase58());
  });

  it("4. Rejects a proposal", async () => {
    await program.methods
      .rejectProposal({
        memo: null,
      })
      .accountsStrict({
        multisig: multisigPda,
        member: owner2.publicKey,
        proposal: proposalPda,
      })
      .signers([owner2])
      .rpc();

    const proposalState = await program.account.proposal.fetch(proposalPda);
    assert.strictEqual(proposalState.rejected.length, 1);
    assert.strictEqual(proposalState.rejected[0].toBase58(), owner2.publicKey.toBase58());
  });
  it("5. Cancels a proposal", async () => {
    await program.methods
      .cancelProposal({
        memo: null,
      })
      .accountsStrict({
        multisig: multisigPda,
        member: owner3.publicKey,
        proposal: proposalPda,
      })
      .signers([owner3])
      .rpc();

    const proposalState = await program.account.proposal.fetch(proposalPda);
    assert.strictEqual(proposalState.cancelled.length, 1);
    assert.strictEqual(proposalState.cancelled[0].toBase58(), owner3.publicKey.toBase58());
  });
});
