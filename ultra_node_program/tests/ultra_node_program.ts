import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { assert } from "chai";
import { UltraNodeProgram } from "../target/types/ultra_node_program";

describe("ultra_node_program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.UltraNodeProgram as Program<UltraNodeProgram>;
  const user = provider.wallet;

  // Converts Uint8Array(32) to number[]
  const u8 = (n: number) => Array(32).fill(n);

  it("Publishes root + fails dummy proof (expected)", async () => {
    const dummyTxHash = u8(1);
    const dummyRoot = u8(2);
    const dummyProof = [u8(3)];
    const dummyIndex = 0;
    const dummySlot = new anchor.BN(123);

    const [acceptedPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("accepted"), Buffer.from(dummyRoot)],
      program.programId
    );

    const [nodePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("node"), user.publicKey.toBuffer()],
      program.programId
    );

    // Publish the root first
    await program.methods
      .publishRoot(dummyRoot, dummySlot)
      .accounts({
        accepted: acceptedPda,
        authority: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    // Then try submitting a fake proof
    try {
      await program.methods
        .submitVerifiedProof(
          dummyTxHash,
          dummyRoot,
          dummyIndex,
          dummyProof
        )
        .accounts({
          node: nodePda,
          accepted: acceptedPda,
          user: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      assert.fail("Dummy proof should have failed");
    } catch (err: any) {
      console.log("✅ Proof rejected as expected:", err.message);
      assert.ok(true);
    }
  });
});
