import { existsSync } from 'node:fs';
import { delimiter, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

// A standard Rust installation takes precedence. This workspace also supports an
// isolated toolchain without changing the user's global PATH or shell profile.
const root = fileURLToPath(new URL('../', import.meta.url));
if (spawnSync('cargo', ['--version'], { windowsHide: true }).error) {
  const localCargo = join(root, '.tools', 'cargo');
  if (existsSync(join(localCargo, 'bin', 'cargo.exe'))) {
    process.env.CARGO_HOME = localCargo;
    process.env.RUSTUP_HOME = join(root, '.tools', 'rustup');
    process.env.PATH = `${join(localCargo, 'bin')}${delimiter}${process.env.PATH ?? ''}`;
  }
}
const { run } = await import('@tauri-apps/cli');
try {
  await run(process.argv.slice(2), 'npm run tauri');
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
