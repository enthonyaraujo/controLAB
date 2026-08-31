const expected = process.argv[2];

if (!expected) {
  console.error('Uso: node scripts/assert-platform.js <linux|win32>');
  process.exit(1);
}

if (process.platform !== expected) {
  const target = expected === 'win32' ? 'Windows' : expected;
  console.error(
    `[LGR Studio] Gere o pacote de ${target} no próprio ${target}. ` +
    'O motor Python do executável é específico de cada sistema operacional.'
  );
  process.exit(1);
}
