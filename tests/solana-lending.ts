import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaLending } from "../target/types/solana_lending";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import assert from "assert";

describe("solana-lending", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.solanaLending as Program<SolanaLending>;
  const connection = provider.connection;

  // Test accounts
  const admin = anchor.web3.Keypair.generate();
  const oracle = anchor.web3.Keypair.generate();
  const borrower = anchor.web3.Keypair.generate();
  let usdcMint: PublicKey;
  let globalStatePDA: PublicKey;
  let treasury: PublicKey;

  before(async () => {
    // Airdrop SOL to test accounts
    await Promise.all([
      connection.requestAirdrop(admin.publicKey, 1000000000),
      connection.requestAirdrop(borrower.publicKey, 1000000000)
    ]);

    // Create USDC mint
    usdcMint = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      6,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID
    );

    // Create treasury account
    treasury = (await getOrCreateAssociatedTokenAccount(
      connection,
      admin,
      usdcMint,
      admin.publicKey
    )).address;

    // Initialize program
    [globalStatePDA] = await PublicKey.findProgramAddressSync(
      [Buffer.from("global_state")],
      program.programId
    );

    await program.methods.initialize(
      100,    // protocol_fee_rate (1%)
      7000,   // ltv_threshold (70%)
      3888000,// min_stake_duration (45 days in seconds)
      oracle.publicKey,
      100000  // oracle_fee (0.1 USDC)
    ).accounts({
      globalState: globalStatePDA,
      admin: admin.publicKey,
      systemProgram: SystemProgram.programId
    }).signers([admin]).rpc();
  });

  describe("Oracle Verification", () => {
    it("should successfully request loan with valid oracle signature", async () => {
      // Setup borrower account
      const borrowerUsdcAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        borrower,
        usdcMint,
        borrower.publicKey
      );

      // Fund borrower with USDC using spl-token
      await mintTo(
        connection,
        admin,
        usdcMint,
        borrowerUsdcAccount.address,
        admin.publicKey,
        1000000 // 1 USDC
      );

      // Create mock oracle signature (simplified for test)
      const signature = new Uint8Array(64).fill(1); // Mock signature

      // Request loan (using the correct method name from IDL)
      await program.methods.requestLoan(
        new anchor.BN(100), // 100 USDC
        75, // credit score
        Array.from(signature) // Convert Uint8Array to number[]
      ).accounts({
        borrower: borrower.publicKey,
        borrowerUsdcAccount: borrowerUsdcAccount.address,
        treasury,
        globalState: globalStatePDA,
        tokenProgram: TOKEN_PROGRAM_ID
      }).signers([borrower]).rpc();
    });

    it("should reject loan with invalid oracle signature", async () => {
      const invalidSignature = new Uint8Array(64).fill(2); // Different mock signature

      await assert.rejects(
        program.methods.requestLoan(
          new anchor.BN(100),
          75,
          Array.from(invalidSignature)
        ).accounts({
          borrower: borrower.publicKey,
          borrowerUsdcAccount: (await getOrCreateAssociatedTokenAccount(
            connection,
            borrower,
            usdcMint,
            borrower.publicKey
          )).address,
          treasury,
          globalState: globalStatePDA,
          tokenProgram: TOKEN_PROGRAM_ID
        }).signers([borrower]).rpc(),
        /InvalidOracleSignature/
      );
    });
  });
});
