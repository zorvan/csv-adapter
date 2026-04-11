const chalk = require('chalk');
const fs = require('fs');
const path = require('path');
const os = require('os');

/**
 * Generate an ASCII art completion certificate
 * @param {object} stats - Certificate statistics
 * @returns {string}
 */
function generateCertificate(stats) {
  const {
    tutorialName,
    completionDate,
    completionTime,
    stepsCompleted,
    badges,
    certificateId
  } = stats;

  const width = 60;

  let cert = '\n';
  cert += chalk.gray('┏') + chalk.gray('━'.repeat(width)) + chalk.gray('┓') + '\n';
  cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';

  // Decorative top
  const topArt = '  ★  ★  ★  ★  ★  ★  ★  ★  ★  ★  ★  ★  ';
  cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - topArt.length) / 2))) +
    chalk.yellowBright(topArt) +
    chalk.gray(' '.repeat(Math.ceil((width - topArt.length) / 2))) +
    chalk.gray('┃') + '\n';

  // Title
  const title = 'CSV ADAPTER';
  cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - title.length) / 2))) +
    chalk.bold.cyan(title) +
    chalk.gray(' '.repeat(Math.ceil((width - title.length) / 2))) +
    chalk.gray('┃') + '\n';

  const subtitle = 'Certificate of Completion';
  cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - subtitle.length) / 2))) +
    chalk.bold.yellow(subtitle) +
    chalk.gray(' '.repeat(Math.ceil((width - subtitle.length) / 2))) +
    chalk.gray('┃') + '\n';

  cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - topArt.length) / 2))) +
    chalk.yellowBright(topArt) +
    chalk.gray(' '.repeat(Math.ceil((width - topArt.length) / 2))) +
    chalk.gray('┃') + '\n';

  cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';

  // Separator
  cert += chalk.gray('┃') + chalk.gray(' ') + chalk.dim('─'.repeat(width - 2)) + chalk.gray(' ') + chalk.gray('┃') + '\n';
  cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';

  // Tutorial name
  const tutLabel = 'Tutorial Completed';
  cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - tutLabel.length) / 2))) +
    chalk.dim(tutLabel) +
    chalk.gray(' '.repeat(Math.ceil((width - tutLabel.length) / 2))) +
    chalk.gray('┃') + '\n';

  cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - tutorialName.length) / 2))) +
    chalk.bold.whiteBright(tutorialName) +
    chalk.gray(' '.repeat(Math.ceil((width - tutorialName.length) / 2))) +
    chalk.gray('┃') + '\n';

  cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';

  // Stats
  const statsLines = [
    ['Date', completionDate],
    ['Duration', completionTime],
    ['Steps', `${stepsCompleted} / ${stepsCompleted}`],
    ['Certificate ID', certificateId]
  ];

  for (const [label, value] of statsLines) {
    const line = `${chalk.dim(label.padEnd(16))}${chalk.cyan(value)}`;
    const visualLen = line.replace(/\x1b\[[0-9;]*m/g, '').length;
    cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - visualLen) / 2))) +
      line +
      chalk.gray(' '.repeat(Math.ceil((width - visualLen) / 2))) +
      chalk.gray('┃') + '\n';
  }

  cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';

  // Badges
  if (badges && badges.length > 0) {
    cert += chalk.gray('┃') + chalk.gray(' ') + chalk.dim('─'.repeat(width - 2)) + chalk.gray(' ') + chalk.gray('┃') + '\n';
    cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';

    const badgeTitle = 'Achievements Unlocked';
    cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - badgeTitle.length) / 2))) +
      chalk.dim(badgeTitle) +
      chalk.gray(' '.repeat(Math.ceil((width - badgeTitle.length) / 2))) +
      chalk.gray('┃') + '\n';

    cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';

    for (const badge of badges) {
      const badgeDisplay = `  🏅 ${badge}`;
      cert += chalk.gray('┃') + chalk.gray('  ') + chalk.yellowBright(badgeDisplay) +
        chalk.gray(' '.repeat(Math.max(0, width - 4 - badgeDisplay.replace(/\x1b\[[0-9;]*m/g, '').length))) +
        chalk.gray('┃') + '\n';
    }

    cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';
  }

  // Bottom decorative art
  const bottomArt = '  ════ CSV ADAPTER ════  ';
  cert += chalk.gray('┃') + chalk.gray(' '.repeat(Math.floor((width - bottomArt.length) / 2))) +
    chalk.cyanBright(bottomArt) +
    chalk.gray(' '.repeat(Math.ceil((width - bottomArt.length) / 2))) +
    chalk.gray('┃') + '\n';

  cert += chalk.gray('┃') + chalk.gray(' '.repeat(width)) + chalk.gray('┃') + '\n';
  cert += chalk.gray('┗') + chalk.gray('━'.repeat(width)) + chalk.gray('┛') + '\n';

  return cert;
}

/**
 * Generate a plain text version of the certificate (for saving to file)
 * @param {object} stats
 * @returns {string}
 */
function generatePlainCertificate(stats) {
  const {
    tutorialName,
    completionDate,
    completionTime,
    stepsCompleted,
    badges,
    certificateId
  } = stats;

  let cert = '';
  cert += '═══════════════════════════════════════════════════════════\n';
  cert += '                    CSV ADAPTER\n';
  cert += '                Certificate of Completion\n';
  cert += '═══════════════════════════════════════════════════════════\n\n';
  cert += `  Tutorial Completed\n`;
  cert += `  ${tutorialName}\n\n`;
  cert += `  Date:           ${completionDate}\n`;
  cert += `  Duration:       ${completionTime}\n`;
  cert += `  Steps:          ${stepsCompleted} / ${stepsCompleted}\n`;
  cert += `  Certificate ID: ${certificateId}\n\n`;

  if (badges && badges.length > 0) {
    cert += '  Achievements Unlocked:\n';
    for (const badge of badges) {
      cert += `    [✓] ${badge}\n`;
    }
    cert += '\n';
  }

  cert += '═══════════════════════════════════════════════════════════\n';
  cert += '  Generated by CSV Adapter Interactive Tutorial\n';
  cert += '═══════════════════════════════════════════════════════════\n';

  return cert;
}

/**
 * Save the certificate to a file
 * @param {object} stats
 * @param {string} [outputPath] - Optional custom path
 * @returns {Promise<string>} Path to the saved file
 */
async function saveCertificate(stats, outputPath) {
  const defaultDir = path.join(os.homedir(), '.csv-tutorial');
  const defaultPath = path.join(defaultDir, `certificate-${stats.certificateId}.txt`);
  const filePath = outputPath || defaultPath;

  // Ensure directory exists
  const dir = path.dirname(filePath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }

  const plainText = generatePlainCertificate(stats);
  fs.writeFileSync(filePath, plainText, 'utf-8');

  return filePath;
}

/**
 * Display the full certificate experience
 * @param {object} renderer - The renderer module
 * @param {object} stats
 * @param {function} waitForUser - Function to wait for user input
 */
async function showCertificate(renderer, stats, waitForUser) {
  console.log('\n');
  console.log(generateCertificate(stats));

  const plainText = generatePlainCertificate(stats);
  const savePath = await saveCertificate(stats);

  console.log(chalk.dim(`  Certificate saved to: ${savePath}\n`));
  console.log(chalk.greenBright('  Congratulations on completing the tutorial!'));
  console.log(chalk.dim('  You now have the knowledge to build cross-chain applications with CSV Adapter.\n'));
}

module.exports = {
  generateCertificate,
  generatePlainCertificate,
  saveCertificate,
  showCertificate
};
