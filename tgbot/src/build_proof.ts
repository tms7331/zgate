// Slightly modified from:
// https://github.com/HorizenLabs/zkverify-example-typescript/blob/main/src/types.ts
import { exec } from "child_process";
import { promisify } from "util";
import * as fs from "fs";
import * as path from "path";

const execAsync = promisify(exec);
interface ProofInner {
    a: string;
    b: string;
    c: string;
}

type ProofType<T> = ProofInner | T | string;

interface Proof<T> {
    curve?: string;
    proof: ProofType<T>;
}

interface ProofData<T> {
    proof: T;
    publicSignals: string | string[];
    vk?: any;
}

interface ProofHandler {
    formatProof(proof: any, publicSignals?: string[]): any;
    formatVk(vkJson: any): any;
    formatPubs(pubs: string[]): any;
    generateProof(inputs: any): Promise<ProofData<any>>;
    // verifyProof(vk: any, proof: any, publicSignals: any): Promise<boolean>;
    // generateUniqueInput(): any;
}

export class Risc0Handler implements ProofHandler {
    formatProof(proof: any): string {
        return JSON.stringify(proof);
    }

    formatVk(vkJson: any): any {
        return vkJson;
    }

    formatPubs(pubs: any[]): string {
        return JSON.stringify(pubs);
    }

    async generateProof(): Promise<ProofData<any>> {
        const inputData = "abcdefg";

        const topLevelDir = path.resolve(__dirname, "../../");
        const binaryPath = path.join(topLevelDir, "bin/host");
        const dataDir = path.join(topLevelDir, "./");

        fs.mkdirSync(dataDir, { recursive: true });

        const command = `${binaryPath} ${inputData}`;
        await execAsync(command);

        /*
        const filepath = path.join(dataDir, "vk.txt");
        const data = fs.readFileSync(filepath, "utf8");
        console.log("GOT DATA", data);
        const proof = "abc";
        const publicSignals = "def";
        const vk = "inj";
        */

        const proof = "";
        const publicSignals = "";
        const vk = "";

        return {
            proof,
            publicSignals,
            vk,
        };
    }

    // async verifyProof(vk: any, proof: any, publicSignals: any): Promise<boolean> {
    //     // TODO: Call native risc0 verifier
    //     return true;
    // }

    // generateUniqueInput(): { a: bigint; b: bigint } {
    //     const randomU64 = (): bigint => {
    //         const high = Math.floor(Math.random() * 0x100000000);
    //         const low = Math.floor(Math.random() * 0x100000000);

    //         return (BigInt(high) << 32n) | BigInt(low);
    //     };

    //     const a = randomU64();
    //     const b = randomU64();

    //     return { a, b };
    // }
}

// export default new Risc0Handler();
// const rc = new Risc0Handler();
// rc.generateProof().catch(console.error);