import { spawn } from 'node:child_process'
import { homedir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
const cli = fileURLToPath(new URL('../node_modules/@tauri-apps/cli/tauri.js', import.meta.url))
const child = spawn(process.execPath, [cli, 'dev'], {
  stdio: 'inherit',
  env: { ...process.env, PATH: `${join(homedir(), '.cargo', 'bin')};${process.env.PATH ?? ''}` },
  cwd: fileURLToPath(new URL('../', import.meta.url))
})
child.on('error', error => { console.error(error.message); process.exitCode = 1 })
child.on('exit', code => { process.exitCode = code ?? 1 })
