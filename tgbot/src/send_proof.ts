// Slightly modified from:
// https://github.com/HorizenLabs/zkverify-example-typescript/blob/main/src/send-proof/index.ts
import { initializeApi, submitProof, loadTextFile } from './helpers';
import { handleTransaction } from './transaction';
import * as path from "path";

const proofTypeToPallet: Record<string, string> = {
    groth16: "settlementGroth16Pallet",
    fflonk: "settlementFFlonkPallet",
    zksync: "settlementZksyncPallet",
    risc0: "settlementRisc0Pallet",
};

export const send_proof = async (): Promise<void> => {
    const proofType = "risc0";
    const skipAttestationArg = 'true';
    const skipAttestation = skipAttestationArg === 'true';

    if (!proofType) {
        throw new Error('Proof type argument is required. Usage: npm run generate:single:proof <proof-type> <skipAttestation>');
    }

    const { api, provider, account } = await initializeApi();

    const topLevelDir = path.resolve(__dirname, "../");

    try {
        console.log(`Generating the proof for ${proofType}`);
        // const { proof, publicSignals, vk } = await generateAndNativelyVerifyProof(proofType);
        // const { proof, publicSignals, vk } = await generateAndNativelyVerifyProof(proofType);
        const proof = await loadTextFile(path.join(topLevelDir, "proof.txt"));
        const publicSignals = await loadTextFile(path.join(topLevelDir, "pub.txt"));
        let vk = await loadTextFile(path.join(topLevelDir, "vk.txt"));

        console.log(`${proofType} Proof generated and natively verified.`);
        console.log("PUBLIC SIGNALS: ", publicSignals);
        console.log("VK: ", vk);

        const proofParams = [
            { 'Vk': vk },
            proof,
            publicSignals
        ];

        const pallet = proofTypeToPallet[proofType.trim()];
        const transaction = submitProof(api, pallet, proofParams);

        const startTime = Date.now();
        const timerRefs = { interval: null as NodeJS.Timeout | null, timeout: null as NodeJS.Timeout | null };

        console.log(`Sending ${proofType} proof to zkVerify for verification...`)
        const result = await handleTransaction(api, transaction, account, proofType, startTime, false, timerRefs, undefined, skipAttestation);

        const elapsedTime = ((Date.now() - startTime) / 1000).toFixed(2);
        console.log(`Sent 1 proof, elapsed time: ${elapsedTime}s, result: ${result.result}, attestationId: ${result.attestationId}`);


    } catch (error) {
        console.error(`Failed to send proof: ${error}`);
    }
    // finally {
    //     if (api) await api.disconnect();
    //     if (provider) await provider.disconnect();
    //     process.exit(0);
    // }
};

// send_proof().catch(console.error);