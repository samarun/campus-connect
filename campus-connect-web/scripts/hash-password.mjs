import bcrypt from "bcryptjs";

const password = process.argv[2];

if (!password || password.length < 12) {
  console.error("Usage: npm run hash-password -- \"a strong password with at least 12 characters\"");
  process.exit(1);
}

const hash = await bcrypt.hash(password, 12);
console.log(hash);
console.error("Paste this into .env inside single quotes, for example:");
console.error(`ADMIN_PASSWORD_HASH='${hash}'`);
