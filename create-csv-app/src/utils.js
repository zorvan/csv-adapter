const chalk = require('chalk');
const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

/**
 * Format an address for display with truncation
 * @param {string} address - Full blockchain address
 * @returns {string} Formatted address (e.g., "0x1234...5678")
 */
function formatAddress(address) {
  if (!address || typeof address !== 'string') return 'N/A';
  if (address.length <= 12) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

/**
 * Validate a project name for filesystem compatibility
 * @param {string} name - Proposed project name
 * @returns {{ valid: boolean, error?: string }} Validation result
 */
function validateProjectName(name) {
  if (!name || name.trim().length === 0) {
    return { valid: false, error: 'Project name cannot be empty' };
  }

  if (name.length > 128) {
    return { valid: false, error: 'Project name must be 128 characters or fewer' };
  }

  if (!/^[a-z0-9][a-z0-9_-]*$/i.test(name)) {
    return {
      valid: false,
      error: 'Project name can only contain letters, numbers, hyphens, and underscores, and must start with a letter or number'
    };
  }

  if (/^[_.\-]/.test(name)) {
    return { valid: false, error: 'Project name cannot start with a dot, underscore, or hyphen' };
  }

  if (/\s/.test(name)) {
    return { valid: false, error: 'Project name cannot contain spaces' };
  }

  // Check for reserved names
  const reservedNames = ['node_modules', 'src', 'test', 'csv-adapter'];
  if (reservedNames.includes(name.toLowerCase())) {
    return { valid: false, error: `"${name}" is a reserved name, please choose another` };
  }

  return { valid: true };
}

/**
 * Check if Node.js version meets minimum requirements
 * @param {string} minVersion - Minimum required version (default: 18.0.0)
 * @returns {{ meetsRequirement: boolean, currentVersion: string }}
 */
function checkNodeVersion(minVersion = '18.0.0') {
  try {
    const version = process.version.replace('v', '');
    const [major] = version.split('.').map(Number);
    const [minMajor] = minVersion.split('.').map(Number);

    return {
      meetsRequirement: major >= minMajor,
      currentVersion: version,
      requiredVersion: minVersion
    };
  } catch (error) {
    return {
      meetsRequirement: false,
      currentVersion: 'unknown',
      requiredVersion: minVersion,
      error: error.message
    };
  }
}

/**
 * Check if Git is installed and available
 * @returns {{ installed: boolean, version?: string }}
 */
function checkGitInstalled() {
  try {
    const version = execSync('git --version', { encoding: 'utf8' }).trim();
    return { installed: true, version };
  } catch (error) {
    return { installed: false };
  }
}

/**
 * Create an ora spinner with standardized styling
 * @param {string} text - Spinner text
 * @returns {object} Ora spinner instance
 */
function createSpinner(text) {
  const ora = require('ora');
  return ora({
    text: chalk.cyan(text),
    color: 'cyan',
    spinner: 'dots'
  });
}

/**
 * Print a formatted section header
 * @param {string} text - Header text
 */
function printHeader(text) {
  console.log('');
  console.log(chalk.bold.cyan('=' .repeat(60)));
  console.log(chalk.bold.cyan(`  ${text}`));
  console.log(chalk.bold.cyan('=' .repeat(60)));
  console.log('');
}

/**
 * Print a formatted success message
 * @param {string} message - Success message
 */
function printSuccess(message) {
  console.log(chalk.green.bold('\n  SUCCESS  ') + chalk.green(message));
}

/**
 * Print a formatted error message
 * @param {string} message - Error message
 * @param {Error?} error - Optional error object
 */
function printError(message, error = null) {
  console.log(chalk.red.bold('\n  ERROR  ') + chalk.red(message));
  if (error && error.message) {
    console.log(chalk.dim(`  ${error.message}`));
  }
}

/**
 * Print a formatted warning message
 * @param {string} message - Warning message
 */
function printWarning(message) {
  console.log(chalk.yellow.bold('\n  WARNING  ') + chalk.yellow(message));
}

/**
 * Print a formatted info message
 * @param {string} message - Info message
 */
function printInfo(message) {
  console.log(chalk.blue.bold('\n  INFO  ') + chalk.blue(message));
}

/**
 * Print step-by-step progress with numbers
 * @param {number} step - Step number
 * @param {string} total - Total steps
 * @param {string} message - Step description
 */
function printStep(step, total, message) {
  console.log(chalk.dim(`  [${step}/${total}]`) + ` ${chalk.white(message)}`);
}

/**
 * Check if a directory exists
 * @param {string} dirPath - Directory path
 * @returns {boolean}
 */
function directoryExists(dirPath) {
  try {
    return fs.existsSync(dirPath) && fs.statSync(dirPath).isDirectory();
  } catch {
    return false;
  }
}

/**
 * Check if a file exists
 * @param {string} filePath - File path
 * @returns {boolean}
 */
function fileExists(filePath) {
  try {
    return fs.existsSync(filePath) && fs.statSync(filePath).isFile();
  } catch {
    return false;
  }
}

/**
 * Format file size for display
 * @param {number} bytes - Size in bytes
 * @returns {string} Formatted size
 */
function formatFileSize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

/**
 * Get the package manager command and install command
 * @param {string} packageManager - Package manager name
 * @returns {{ run: string, install: string, exec: string }}
 */
function getPackageManagerCommands(packageManager) {
  const commands = {
    npm: { run: 'npm run', install: 'npm install', exec: 'npx' },
    yarn: { run: 'yarn', install: 'yarn add', exec: 'yarn' },
    pnpm: { run: 'pnpm run', install: 'pnpm add', exec: 'pnpm' },
    bun: { run: 'bun run', install: 'bun add', exec: 'bunx' }
  };
  return commands[packageManager] || commands.npm;
}

/**
 * Sanitize a string for use in shell commands
 * @param {string} input - Input string
 * @returns {string} Sanitized string
 */
function sanitizeShellInput(input) {
  return String(input).replace(/[^a-zA-Z0-9_\-\.\/]/g, '');
}

/**
 * Delay execution for specified milliseconds
 * @param {number} ms - Milliseconds to delay
 * @returns {Promise<void>}
 */
function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

module.exports = {
  formatAddress,
  validateProjectName,
  checkNodeVersion,
  checkGitInstalled,
  createSpinner,
  printHeader,
  printSuccess,
  printError,
  printWarning,
  printInfo,
  printStep,
  directoryExists,
  fileExists,
  formatFileSize,
  getPackageManagerCommands,
  sanitizeShellInput,
  delay
};
