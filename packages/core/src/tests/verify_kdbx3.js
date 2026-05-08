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
console.log('IV:', iv.toString('hex'));
console.log('StreamStart:', streamStart.toString('hex'));
console.log('Header ends at pos:', pos);

const pwHash = crypto.createHash('sha256').update('test123').digest();
const compositeKey = crypto.createHash('sha256').update(pwHash).digest();
let half1 = Buffer.from(compositeKey.slice(0, 16)),
  half2 = Buffer.from(compositeKey.slice(16, 32));
for (let i = 0; i < tRounds; i++) {
  const c1 = crypto.createCipheriv('aes-256-ecb', tSeed, null);
  c1.setAutoPadding(false);
  half1 = c1.update(half1);
  const c2 = crypto.createCipheriv('aes-256-ecb', tSeed, null);
  c2.setAutoPadding(false);
  half2 = c2.update(half2);
}
const transformedKey = crypto
  .createHash('sha256')
  .update(Buffer.concat([half1, half2]))
  .digest();
const encKey = crypto.createHash('sha256').update(mSeed).update(transformedKey).digest();
console.log('encKey:', encKey.toString('hex'));

const ciphertext = data.slice(pos + 32); // skip 32-byte header hash
const decipher = crypto.createDecipheriv('aes-256-cbc', encKey, iv);
decipher.setAutoPadding(false);
try {
  const decrypted = Buffer.concat([decipher.update(ciphertext), decipher.final()]);
  console.log('Decrypted first 32:', decrypted.slice(0, 32).toString('hex'));
  console.log('StreamStart:       ', streamStart.toString('hex'));
  console.log('Match:', decrypted.slice(0, 32).toString('hex') === streamStart.toString('hex'));
} catch (e) {
  console.log('Decrypt error:', e.message);
}
