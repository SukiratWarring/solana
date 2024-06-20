import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Checking } from "../target/types/checking";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  getAccount,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { assert } from "chai";

describe("checking", () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);
  const payer = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.Checking as Program<Checking>;

  const connection = new Connection("http://127.0.0.1:8899", "confirmed");

  const mintKeyPair = Keypair.fromSecretKey(
    new Uint8Array([
      230, 183, 240, 37, 196, 151, 190, 13, 114, 196, 237, 244, 179, 188, 28,
      139, 153, 245, 145, 175, 254, 94, 37, 58, 84, 1, 248, 133, 85, 53, 212,
      82, 202, 221, 224, 217, 38, 3, 63, 75, 125, 195, 27, 109, 107, 194, 184,
      146, 190, 103, 164, 69, 185, 251, 208, 116, 154, 71, 21, 147, 144, 227,
      68, 95,
    ])
  );
  console.log("mintKeyPair", mintKeyPair);

  /**
   * Create and initialize a new mint
   *
   * @param connection      Connection to use
   * @param payer           Payer of the transaction and initialization fees
   * @param mintAuthority   Account or multisig that will control minting
   * @param freezeAuthority Optional account or multisig that can freeze token accounts
   * @param decimals        Location of the decimal place
   * @param keypair         Optional keypair, defaulting to a new random one
   * @param confirmOptions  Options for confirming the transaction
   * @param programId       SPL Token program account
   *
   */
  async function createMintToken() {
    const mint = await createMint(
      connection,
      payer.payer,
      payer.publicKey,
      payer.publicKey,
      9,
      mintKeyPair
    );
    console.log("mint", mint);
  }
  it("Initializing", async () => {
    // Add your test here.
    // await createMintToken();
    const [vault_account] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault")],
      program.programId
    );

    const tx = await program.methods
      .initialize()
      .accounts({
        signer: payer.publicKey,
        mint: mintKeyPair.publicKey,
      })
      .rpc();
    console.log("Your transaction signature", tx);
  });

  it("Stake", async () => {
    // User_ATA
    const user_ata = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeyPair.publicKey,
      payer.publicKey
    );
    //Minting it to the newly created User_ATA
    // await mintTo(
    //   connection,
    //   payer.payer,
    //   mintKeyPair.publicKey,
    //   user_ata.address,
    //   payer.payer,
    //   1000_000_000_000
    // );

    const [user_stake_account] = PublicKey.findProgramAddressSync(
      [Buffer.from("token")],
      program.programId
    );

    const [user_stake_info] = PublicKey.findProgramAddressSync(
      [Buffer.from("token"), payer.publicKey.toBuffer()],
      program.programId
    );
    // {
    //   userTokenAccount: user_ata.address,
    //   mintAccount: mintKeyPair.publicKey,
    //   signer: payer.publicKey,
    //   stake_info_account,
    // }
    const tx = await program.methods
      .stake(new anchor.BN(1))
      .accounts({
        userTokenAccount: user_ata.address,
        mintAccount: mintKeyPair.publicKey,
        signer: payer.publicKey,
      })
      .rpc();
    console.log("tx", tx);
  });

  it("Check user_account_info & stake_account after stake", async () => {
    const [user_account_info] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake_info"), payer.publicKey.toBuffer()],
      program.programId
    );
    console.log("user_account_info", user_account_info);
    const accountData = await program.account.stakeInfo.fetch(
      user_account_info
    );
    assert.ok(accountData, "StakeInfo account should exist");
    assert.strictEqual(
      accountData.isStaked,
      true,
      "The account should have staked"
    );

    console.log("accountData", accountData);
  });

  it("USER ATA details", async () => {
    const ata = await getOrCreateAssociatedTokenAccount(
      connection,
      payer.payer,
      mintKeyPair.publicKey,
      payer.publicKey
    );
    console.log("ata", ata);
    assert.strictEqual(
      Number(ata.amount),
      8999000000000,
      "The account should have staked"
    );
  });

  it("PDA+TOKENACCOUNT details", async () => {
    // Find the PDA for the stake account
    const [stakeAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("token"), payer.publicKey.toBuffer()],
      program.programId
    );
    const stakeAccountData = await getAccount(
      provider.connection,
      stakeAccount
    );
    console.log("stakeAccountData", stakeAccountData);
  });
});
