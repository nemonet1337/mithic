const fs = require('fs');
const { execSync } = require('child_process');

const cwd = process.cwd();

function run(cmd) {
  return execSync(cmd, { encoding: 'utf8', cwd, maxBuffer: 50 * 1024 * 1024 });
}

// 1. git status --short
const status = run('git status --short').trimEnd();
console.log('=== SHORT STATUS ===');
console.log(status);
console.log('=== END SHORT STATUS ===');

// 2. git diff HEAD --stat
const diffstat = run('git -c core.quotepath=false diff HEAD --stat').trimEnd();
console.log('=== DIFF STAT ===');
console.log(diffstat);
console.log('=== END DIFF STAT ===');

// 3. git diff HEAD (full)
const diff = run('git -c core.quotepath=false diff HEAD').trimEnd();
console.log('=== FULL DIFF ===');
console.log(diff);
console.log('=== END FULL DIFF ===');

// 4. git ls-files --others --exclude-standard
const untracked = run('git ls-files --others --exclude-standard').trimEnd();
console.log('=== UNTRACKED FILES ===');
console.log(untracked);
console.log('=== END UNTRACKED FILES ===');
