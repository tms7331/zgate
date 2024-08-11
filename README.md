# zgate

**zgate** is a Telegram-based access control system that leverages zero-knowledge proofs to securely gate access to private Telegram chats. The project combines Risc Zero (https://dev.risczero.com/api/) and zkVerify (https://docs.zkverify.io/) to ensure that users can prove their onchain activity without revealing their private keys.

zgate was built for zkhack Montreal.

## How It Works

1. **Interaction with the Telegram Bot**  
   Users begin by interacting with the Telegram bot located at [t.me/zgated_bot](https://t.me/zgated_bot). The bot provides the user with a challenge message which includes the user's Telegram ID, which a user must sign with their private key. 

2. **Message Signing**  
   The user signs the message provided by the bot using their private key. This allows them to prove ownership of the public key without needing to reveal the private key.

3. **Submitting the Data**  
   The signed message is submitted back to the Telegram bot. The bot then forwards this data to a Risc Zero prover, which runs on the same server as the bot.

4. **Zero-Knowledge Proof Generation and Verification**  
   The Risc Zero prover generates a zero-knowledge proof (ZKP) based on the signed message. This proof is then verified and published on-chain using the zkVerify protocol. zkVerify ensures the proof's validity without compromising the user's private information.

5. **Access Granted**  
   Once the proof is successfully verified and published on-chain, the Telegram bot generates a single-use join link for a private chat. This link allows the user to access the gated Telegram chat securely.

## Technologies Used

- **Risc Zero**: A zero-knowledge proof system that ensures secure and private computation.
- **zkVerify**: A protocol that verifies zero-knowledge proofs and publishes them on-chain, enabling trustless verification.
- **Telegram Bot API**: For interacting with users and managing access to Telegram chats.

## Getting Started

1. **Start by interacting with the bot**: [t.me/zgated_bot](https://t.me/zgated_bot).
2. Follow the bot's instructions to sign a message with your private key.
3. Submit the signed message to the bot.
4. Receive a single-use join link to access the private Telegram chat.

## License

This project is licensed under the MIT License.

