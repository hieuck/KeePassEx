// Test with "abc" password vault
const crypto = require('crypto');
const fs = require('fs');

function testVault(vaultPath, password) {
  const data = fs.readFileSync(vaultPath);
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
  console.log('Vault:', vaultPath, 'password:', password, 'rounds:', tRounds);

  // Standard KeePass: compositeKey = SHA256(SHA256(password))
  const ck = crypto
    .createHash('sha256')
    .update(crypto.createHash('sha256').update(password).digest())
    .digest();

  let h1 = Buffer.from(ck.slice(0, 16));
  let h2 = Buffer.from(ck.slice(16, 32));
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
    console.log('  match:', match);
    console.log('  expected:', streamStart.toString('hex'));
    console.log('  got:     ', dec.slice(0, 32).toString('hex'));
  } catch (e) {
    console.log('  error:', e.message);
  }
}

testVault('packages/core/src/tests/test_kpxc.kdbx', 'test123');
testVault('packages/core/src/tests/test_abc.kdbx', 'abc');
