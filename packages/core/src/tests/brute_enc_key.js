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

function tryKey(label, compositeKey) {
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
  const tk = crypto
    .createHash('sha256')
    .update(Buffer.concat([h1, h2]))
    .digest();
  const ek = crypto.createHash('sha256').update(mSeed).update(tk).digest();
  const d = crypto.createDecipheriv('aes-256-cbc', ek, iv);
  d.setAutoPadding(false);
  try {
    const dec = Buffer.concat([d.update(data.slice(pos + 32)), d.final()]);
    const match = dec.slice(0, 32).toString('hex') === streamStart.toString('hex');
    console.log(label + ': match=' + match + ' ek=' + ek.toString('hex').slice(0, 16) + '...');
    if (match) console.log('  FOUND! compositeKey=' + compositeKey.toString('hex'));
  } catch (e) {
    console.log(label + ': decrypt error');
  }
}

// Try 1: SHA256(password) — single hash
const ck1 = crypto.createHash('sha256').update('test123').digest();
tryKey('single_hash', ck1);

// Try 2: SHA256(SHA256(password)) — double hash (KeePass standard)
const ck2 = crypto.createHash('sha256').update(ck1).digest();
tryKey('double_hash', ck2);

// Try 3: raw password bytes
const ck3 = Buffer.from('test123', 'utf8');
if (ck3.length === 32) tryKey('raw_password', ck3);

// Try 4: SHA256(password_utf16le)
const ck4 = crypto.createHash('sha256').update(Buffer.from('test123', 'utf16le')).digest();
tryKey('sha256_utf16le', ck4);

// Try 5: SHA256(SHA256(password_utf16le))
const ck5 = crypto.createHash('sha256').update(ck4).digest();
tryKey('double_sha256_utf16le', ck5);
