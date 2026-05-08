// Try to find the correct enc_key by trying all KeePass key derivation variants
const crypto = require('crypto');
const fs = require('fs');

const data = fs.readFileSync('packages/core/src/tests/test_kpxc.kdbx');
let pos = 12,
  tSeed,
  tRounds,
  mSeed,
  iv,
  streamStart;
while (pos < data.length) {
  const fId = data[pos],
    fLen = data.readUInt16LE(pos + 1),
    fData = data.slice(pos + 3, pos + 3 + fLen);
  if (fId === 0) {
    pos += 3 + fLen;
    break;
  }
  if (fId === 4) mSeed = fData;
  if (fId === 5) tSeed = fData;
  if (fId === 6) tRounds = Number(fData.readBigUInt64LE(0));
  if (fId === 7) iv = fData;
  if (fId === 9) streamStart = fData;
  pos += 3 + fLen;
}

const ciphertext = data.slice(pos + 32);
console.log('ciphertext len:', ciphertext.length);
console.log('streamStart:', streamStart.toString('hex'));

function aesKdf(compositeKey) {
  let h1 = Buffer.from(compositeKey.slice(0, 16));
  let h2 = Buffer.from(compositeKey.slice(16, 32));
  for (let i = 0; i < tRounds; i++) {
    const c1 = crypto.createCipheriv('aes-256-ecb', tSeed, null);
    c1.setAutoPadding(false);
    h1 = c1.update(h1);
    const c2 = crypto.createCipheriv('aes-256-ecb', tSeed, null);
    c2.setAutoPadding(false);
    h2 = c2.update(h2);
  }
  return crypto
    .createHash('sha256')
    .update(Buffer.concat([h1, h2]))
    .digest();
}

function tryDecrypt(ek) {
  const d = crypto.createDecipheriv('aes-256-cbc', ek, iv);
  d.setAutoPadding(false);
  try {
    const dec = Buffer.concat([d.update(ciphertext), d.final()]);
    return dec.slice(0, 32).toString('hex') === streamStart.toString('hex');
  } catch (e) {
    return false;
  }
}

const password = 'test123';

// Variant 1: KeePass 2.x standard
// compositeKey = SHA256(SHA256(password))
// transformedKey = AES-KDF(compositeKey)
// encKey = SHA256(masterSeed || transformedKey)
const v1_ck = crypto
  .createHash('sha256')
  .update(crypto.createHash('sha256').update(password).digest())
  .digest();
const v1_tk = aesKdf(v1_ck);
const v1_ek = crypto.createHash('sha256').update(mSeed).update(v1_tk).digest();
console.log('v1 (standard):', tryDecrypt(v1_ek), 'ek:', v1_ek.toString('hex').slice(0, 16));

// Variant 2: compositeKey = SHA256(password) (single hash)
const v2_ck = crypto.createHash('sha256').update(password).digest();
const v2_tk = aesKdf(v2_ck);
const v2_ek = crypto.createHash('sha256').update(mSeed).update(v2_tk).digest();
console.log('v2 (single hash):', tryDecrypt(v2_ek), 'ek:', v2_ek.toString('hex').slice(0, 16));

// Variant 3: KeePass 1.x style — SHA256(password_bytes) directly as key
// No AES-KDF, just SHA256(masterSeed || SHA256(password))
const v3_ek = crypto
  .createHash('sha256')
  .update(mSeed)
  .update(crypto.createHash('sha256').update(password).digest())
  .digest();
console.log('v3 (no kdf):', tryDecrypt(v3_ek), 'ek:', v3_ek.toString('hex').slice(0, 16));

// Variant 4: Use 1 round of AES-KDF
const v4_ck = crypto
  .createHash('sha256')
  .update(crypto.createHash('sha256').update(password).digest())
  .digest();
let v4_h1 = Buffer.from(v4_ck.slice(0, 16)),
  v4_h2 = Buffer.from(v4_ck.slice(16, 32));
const v4_c1 = crypto.createCipheriv('aes-256-ecb', tSeed, null);
v4_c1.setAutoPadding(false);
v4_h1 = v4_c1.update(v4_h1);
const v4_c2 = crypto.createCipheriv('aes-256-ecb', tSeed, null);
v4_c2.setAutoPadding(false);
v4_h2 = v4_c2.update(v4_h2);
const v4_tk = crypto
  .createHash('sha256')
  .update(Buffer.concat([v4_h1, v4_h2]))
  .digest();
const v4_ek = crypto.createHash('sha256').update(mSeed).update(v4_tk).digest();
console.log('v4 (1 round):', tryDecrypt(v4_ek), 'ek:', v4_ek.toString('hex').slice(0, 16));

// Variant 5: No final SHA256 on AES-KDF output
const v5_ck = crypto
  .createHash('sha256')
  .update(crypto.createHash('sha256').update(password).digest())
  .digest();
let v5_h1 = Buffer.from(v5_ck.slice(0, 16)),
  v5_h2 = Buffer.from(v5_ck.slice(16, 32));
for (let i = 0; i < tRounds; i++) {
  const c1 = crypto.createCipheriv('aes-256-ecb', tSeed, null);
  c1.setAutoPadding(false);
  v5_h1 = c1.update(v5_h1);
  const c2 = crypto.createCipheriv('aes-256-ecb', tSeed, null);
  c2.setAutoPadding(false);
  v5_h2 = c2.update(v5_h2);
}
const v5_tk_raw = Buffer.concat([v5_h1, v5_h2]); // no SHA256
const v5_ek = crypto.createHash('sha256').update(mSeed).update(v5_tk_raw).digest();
console.log('v5 (no final sha256):', tryDecrypt(v5_ek), 'ek:', v5_ek.toString('hex').slice(0, 16));
