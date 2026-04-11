const figlet = require('figlet');
const chalk = require('chalk');

const isTTY = process.stdout.isTTY;

/**
 * Render an ASCII art header using figlet
 * @param {string} title - The title text
 * @param {object} [options] - Rendering options
 * @param {string} [options.font='Standard'] - Figlet font name
 * @param {string} [options.color='cyan'] - Chalk color
 * @returns {Promise<string>}
 */
async function renderHeader(title, options = {}) {
  const { font = 'Standard', color = 'cyan' } = options;

  return new Promise((resolve, reject) => {
    figlet(text(title), { font, horizontalLayout: 'default', width: 80 }, (err, data) => {
      if (err) {
        reject(err);
        return;
      }
      resolve(chalk[color](data));
    });
  });
}

/**
 * Render a simple centered text header (fallback for small terminals)
 * @param {string} title
 * @returns {string}
 */
function text(title) {
  return title;
}

/**
 * Render a step display with progress bar
 * @param {object} step - The step object
 * @param {number} current - Current step number (1-indexed)
 * @param {number} total - Total number of steps
 * @returns {string}
 */
function renderStep(step, current, total) {
  const progressBar = renderProgressBar(current, total);
  const divider = chalk.gray('━'.repeat(60));

  let output = '';
  output += `\n${divider}\n`;
  output += `${progressBar}\n`;
  output += `${divider}\n`;
  output += `\n${chalk.bold.yellow('>>')} ${chalk.bold.white(step.title)}\n`;
  output += `${divider}\n`;

  if (step.description) {
    output += `\n${chalk.cyan(step.description)}\n`;
  }

  if (step.concept) {
    output += `\n${chalk.gray('┌' + '─'.repeat(56) + '┐')}\n`;
    output += chalk.gray('│') + chalk.bgBlue(' ' + chalk.bold(' CONCEPT ') + ' ') +
      chalk.gray(' '.repeat(Math.max(0, 56 - 10 - step.concept.length))) +
      chalk.gray(step.concept.length > 46 ? step.concept.slice(0, 46) + '…' : step.concept) +
      chalk.gray('│'.repeat(1)) + '\n';
    // Simplified concept box
    output = output.replace(
      /│.+│\n$/,
      (match) => {
        const conceptText = step.concept.length > 44 ? step.concept.slice(0, 44) + '…' : step.concept;
        const padding = 54 - conceptText.length;
        return chalk.gray('│') + ' ' + chalk.blueBright(conceptText) + ' '.repeat(padding) + chalk.gray('│') + '\n';
      }
    );
    output += chalk.gray('└' + '─'.repeat(56) + '┘\n');
  }

  return output;
}

/**
 * Render a progress bar
 * @param {number} current - Current step
 * @param {number} total - Total steps
 * @param {number} [width=40] - Bar width
 * @returns {string}
 */
function renderProgressBar(current, total, width = 40) {
  const filled = Math.round((current / total) * width);
  const empty = width - filled;
  const percentage = Math.round((current / total) * 100);

  const bar = chalk.green('█'.repeat(filled)) + chalk.gray('░'.repeat(empty));
  return `  [${bar}] ${percentage}%  (${current}/${total})`;
}

/**
 * Render a code block with syntax highlighting simulation
 * @param {string} code - The code to display
 * @param {string} [language=''] - Language label
 * @returns {string}
 */
function renderCode(code, language = '') {
  const langLabel = language ? ` ${language} ` : '';
  const lines = code.trim().split('\n');
  const boxWidth = Math.max(58, ...lines.map(l => l.length + 6));

  let output = '';
  output += chalk.gray('┌' + '─'.repeat(boxWidth - 2) + '┐') + '\n';
  if (langLabel) {
    output += chalk.gray('│') + chalk.bgGray(langLabel.padEnd(boxWidth - 2)) + chalk.gray('│') + '\n';
    output += chalk.gray('├' + '─'.repeat(boxWidth - 2) + '┤') + '\n';
  }
  for (const line of lines) {
    const padded = line.padEnd(boxWidth - 2);
    output += chalk.gray('│') + ' ' + chalk.greenBright(line) + ' '.repeat(boxWidth - 2 - line.length) + chalk.gray('│') + '\n';
  }
  output += chalk.gray('└' + '─'.repeat(boxWidth - 2) + '┘');
  return output;
}

/**
 * Render a result display
 * @param {object|string} result - Result data or message
 * @returns {string}
 */
function renderResult(result) {
  let output = `\n${chalk.bgGreen.black(' RESULT ')}\n`;
  output += chalk.gray('┌' + '─'.repeat(54) + '┐') + '\n';

  if (typeof result === 'string') {
    const padded = result.padEnd(54);
    output += chalk.gray('│') + ' ' + chalk.green(padded.slice(0, 54)) + chalk.gray('│') + '\n';
  } else if (typeof result === 'object') {
    for (const [key, value] of Object.entries(result)) {
      const label = chalk.dim(key.replace(/_/g, ' ') + ':');
      let displayValue;
      if (typeof value === 'boolean') {
        displayValue = value ? chalk.green('true') : chalk.red('false');
      } else if (typeof value === 'number') {
        displayValue = chalk.yellow(value.toString());
      } else {
        displayValue = chalk.cyan(String(value));
      }
      const line = `  ${label} ${displayValue}`;
      output += chalk.gray('│') + chalk.white(line.padEnd(54)) + chalk.gray('│') + '\n';
    }
  }

  output += chalk.gray('└' + '─'.repeat(54) + '┘');
  return output;
}

/**
 * Render a pro tip callout
 * @param {string} tip - The tip text
 * @returns {string}
 */
function renderProTip(tip) {
  const line = chalk.gray('─'.repeat(56));
  return `\n${chalk.gray('┌')}${line}${chalk.gray('┐')}\n` +
    `${chalk.gray('│')}  ${chalk.yellowBright('💡 Pro Tip:')} ${chalk.whiteBright(tip).padEnd(44)}${chalk.gray('│')}\n` +
    `${chalk.gray('└')}${line}${chalk.gray('┘')}\n`;
}

/**
 * Render a warning callout
 * @param {string} message
 * @returns {string}
 */
function renderWarning(message) {
  const line = chalk.gray('─'.repeat(56));
  return `\n${chalk.gray('┌')}${line}${chalk.gray('┐')}\n` +
    `${chalk.gray('│')}  ${chalk.redBright('⚠️  ')}${chalk.yellowBright(message).padEnd(44)}${chalk.gray('│')}\n` +
    `${chalk.gray('└')}${line}${chalk.gray('┘')}\n`;
}

/**
 * Render a phase indicator for multi-phase operations
 * @param {string} phaseName
 * @param {number} current
 * @param {number} total
 * @returns {string}
 */
function renderPhase(phaseName, current, total) {
  const indicator = chalk.dim(`[${current + 1}/${total}]`);
  return `\n  ${indicator} ${chalk.cyan(phaseName)}...`;
}

/**
 * Render a success checkmark
 * @param {string} message
 * @returns {string}
 */
function renderSuccess(message) {
  return `  ${chalk.green('✓')} ${chalk.greenBright(message)}`;
}

/**
 * Render a failure cross
 * @param {string} message
 * @returns {string}
 */
function renderFailure(message) {
  return `  ${chalk.red('✗')} ${chalk.redBright(message)}`;
}

/**
 * Render a divider line
 * @returns {string}
 */
function renderDivider() {
  return chalk.gray('━'.repeat(60));
}

/**
 * Render a sub-section header
 * @param {string} title
 * @returns {string}
 */
function renderSection(title) {
  return `\n${chalk.bold.yellow('▸ ' + title)}\n`;
}

/**
 * Render a list item
 * @param {string} text
 * @param {number} [index] - Optional index for numbered list
 * @returns {string}
 */
function renderListItem(text, index) {
  if (index !== undefined) {
    return `  ${chalk.dim(index + 1 + '.')}. ${text}`;
  }
  return `  ${chalk.dim('•')} ${text}`;
}

/**
 * Render a key-value pair
 * @param {string} key
 * @param {string} value
 * @returns {string}
 */
function renderKV(key, value) {
  return `  ${chalk.dim(key + ':')} ${chalk.cyan(value)}`;
}

/**
 * Render the completion certificate
 * @param {object} stats - Certificate statistics
 * @returns {string}
 */
function renderCertificate(stats) {
  const {
    tutorialName,
    completionDate,
    completionTime,
    stepsCompleted,
    badges,
    certificateId
  } = stats;

  const width = 58;
  const border = '═'.repeat(width);
  const thinBorder = '─'.repeat(width);

  let cert = '\n';
  cert += chalk.gold('╔' + border + '╗') + '\n';
  cert += chalk.gold('║') + chalk.gray(' '.repeat(width)) + chalk.gold('║') + '\n';

  // Title
  const title = '★ CERTIFICATE OF COMPLETION ★';
  cert += chalk.gold('║') + chalk.gray(' '.repeat(Math.floor((width - title.length) / 2))) +
    chalk.bold.yellow(title) +
    chalk.gray(' '.repeat(Math.ceil((width - title.length) / 2))) +
    chalk.gold('║') + '\n';

  cert += chalk.gold('║') + chalk.gray(' '.repeat(width)) + chalk.gold('║') + '\n';
  cert += chalk.gold('║') + chalk.gray(thinBorder) + chalk.gold('║') + '\n';
  cert += chalk.gold('║') + chalk.gray(' '.repeat(width)) + chalk.gold('║') + '\n';

  // Tutorial name
  const tutLine = `Tutorial: ${tutorialName}`;
  cert += chalk.gold('║') + chalk.gray(' '.repeat(Math.floor((width - tutLine.length) / 2))) +
    chalk.whiteBright(tutLine) +
    chalk.gray(' '.repeat(Math.ceil((width - tutLine.length) / 2))) +
    chalk.gold('║') + '\n';

  cert += chalk.gold('║') + chalk.gray(' '.repeat(width)) + chalk.gold('║') + '\n';

  // Stats
  const stats_ = [
    ['Date', completionDate],
    ['Time', completionTime],
    ['Steps', `${stepsCompleted} completed`],
    ['ID', certificateId]
  ];

  for (const [label, value] of stats_) {
    const line = `${chalk.dim(label + ':')} ${chalk.cyan(value)}`;
    cert += chalk.gold('║') + chalk.gray(' '.repeat(Math.floor((width - line.replace(/\x1b\[[0-9;]*m/g, '').length) / 2))) +
      line +
      chalk.gray(' '.repeat(Math.ceil((width - line.replace(/\x1b\[[0-9;]*m/g, '').length) / 2))) +
      chalk.gold('║') + '\n';
  }

  cert += chalk.gold('║') + chalk.gray(' '.repeat(width)) + chalk.gold('║') + '\n';

  // Badges
  if (badges && badges.length > 0) {
    const badgesLine = `Badges: ${badges.join(' | ')}`;
    cert += chalk.gold('║') + chalk.gray(' '.repeat(Math.floor((width - badgesLine.replace(/\x1b\[[0-9;]*m/g, '').length) / 2))) +
      chalk.yellowBright(badgesLine) +
      chalk.gray(' '.repeat(Math.ceil((width - badgesLine.replace(/\x1b\[[0-9;]*m/g, '').length) / 2))) +
      chalk.gold('║') + '\n';
    cert += chalk.gold('║') + chalk.gray(' '.repeat(width)) + chalk.gold('║') + '\n';
  }

  cert += chalk.gold('╚' + border + '╝') + '\n';

  return cert;
}

/**
 * Clear the terminal screen
 */
function clearScreen() {
  if (isTTY) {
    process.stdout.write('\x1Bc');
  }
}

/**
 * Render a loading spinner message
 * @param {string} message
 * @param {function} action - Async action to perform while "spinning"
 * @returns {Promise<*>}
 */
async function withSpinner(message, action) {
  const frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
  let i = 0;

  process.stdout.write(`\n  ${chalk.cyan(frames[0])} ${message}`);

  const interval = setInterval(() => {
    i = (i + 1) % frames.length;
    process.stdout.write(`\r  ${chalk.cyan(frames[i])} ${message}`);
  }, 80);

  try {
    const result = await action();
    process.stdout.write(`\r  ${chalk.green('✓')} ${chalk.greenBright(message)}\n`);
    return result;
  } catch (err) {
    process.stdout.write(`\r  ${chalk.red('✗')} ${chalk.redBright(message)}\n`);
    throw err;
  } finally {
    clearInterval(interval);
  }
}

module.exports = {
  renderHeader,
  renderStep,
  renderProgressBar,
  renderCode,
  renderResult,
  renderProTip,
  renderWarning,
  renderPhase,
  renderSuccess,
  renderFailure,
  renderDivider,
  renderSection,
  renderListItem,
  renderKV,
  renderCertificate,
  clearScreen,
  withSpinner
};
