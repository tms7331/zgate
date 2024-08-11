import { Bot, Context } from "grammy";
import { Risc0Handler } from "./build_proof";
import { send_proof } from "./send_proof";
import * as dotenv from 'dotenv';
dotenv.config();

const botKey = process.env.BOT_KEY;
const bot = new Bot(botKey!);

bot.command("start", (ctx: Context) => {
  ctx.reply(
    "Welcome! First, call the /get_message command to get the message that you'll need to sign. " +
    "Sign it here: https://etherscan.io/verifiedSignatures, " +
    "then submit the signed message with the /submit <signed message> command."
  );
});


bot.command("get_message", (ctx: Context) => {
  const userId = ctx.from?.id;
  if (userId) {
    const message = `${userId}zgate`;
    ctx.reply(
      `Please sign the following message with your wallet: "${message}"`
    );
  } else {
    ctx.reply("Unable to retrieve your Telegram ID.");
  }
});


bot.command("submit", async (ctx: Context) => {
  const messageText = ctx.message?.text;

  const signedMessage = messageText!.split(" ")[1];
  // Building proof
  await ctx.reply("Generating risc zero proof...");
  const rc = new Risc0Handler();
  await rc.generateProof().catch(console.error);
  await ctx.reply("Proof generation successful!  Sending proof to zkVerify...");
  await send_proof();

  // chatId for our private chat that users can get access to
  // retrieved via: console.log("CHAT ID", ctx.chat!.id);
  const chatId = -4263833258;
  const inviteLink = await ctx.api.createChatInviteLink(chatId, {
    member_limit: 1,
  });

  await ctx.reply(`Proof verification successful!  Here is your invite link: ${inviteLink.invite_link}`);

});

bot.start();
