#!/usr/bin/env node

const { Command } = require('commander');
const chalk = require('chalk');
const { TutorialRunner } = require('./engine');
const renderer = require('./renderer');
const { showCertificate } = require('./certificate');
const { generateCertificateData } = require('./utils');

// Available tutorials
const tutorials = {
  'cross-chain-basics': require('./tutorials/cross-chain-basics'),
  'nft-transfer': require('./tutorials/nft-transfer'),
  'advanced-proofs': require('./tutorials/advanced-proofs')
};

const program = new Command();

program
  .name('csv-tutorial')
  .description('Interactive CLI tutorial for CSV Adapter - Learn cross-chain transfers step by step')
  .version('1.0.0')
  .argument('[tutorial]', 'Tutorial to run', 'cross-chain-basics')
  .option('-r, --restart', 'Restart tutorial from beginning (ignore saved progress)')
  .option('--non-interactive', 'Run without waiting for user input')
  .option('--list', 'List available tutorials')
  .action(async (tutorialName, options) => {
    // Handle --list flag
    if (options.list) {
      listTutorials();
      return;
    }

    // Resolve tutorial
    const tutorial = tutorials[tutorialName];
    if (!tutorial) {
      console.error(chalk.red(`\n  Error: Unknown tutorial "${tutorialName}"\n`));
      console.log(chalk.white('  Available tutorials:\n'));
      listTutorials();
      process.exit(1);
    }

    // If restart option, clear saved progress
    if (options.restart) {
      const fs = require('fs');
      const path = require('path');
      const os = require('os');
      const progressPath = path.join(os.homedir(), '.csv-tutorial', 'progress', `${tutorial.id}.json`);
      if (fs.existsSync(progressPath)) {
        fs.unlinkSync(progressPath);
        console.log(chalk.yellow(`\n  Cleared saved progress for "${tutorialName}"\n`));
      }
    }

    // Create runner
    const runner = new TutorialRunner(renderer, {
      interactive: !options.nonInteractive
    });

    try {
      // Run the tutorial
      const stats = await runner.run(tutorial);

      // Show certificate
      console.log('\n');
      await showCertificate(renderer, stats, () => Promise.resolve());

    } catch (error) {
      console.error(chalk.red('\n  Tutorial error:'), error.message);
      if (error.stack && process.env.DEBUG) {
        console.error(chalk.dim(error.stack));
      }
      process.exit(1);
    }
  });

program.parse();

/**
 * List available tutorials
 */
function listTutorials() {
  const entries = Object.entries(tutorials);
  const maxNameLen = Math.max(...entries.map(([name]) => name.length));

  for (const [name, tutorial] of entries) {
    const steps = tutorial.steps.length;
    const padded = name.padEnd(maxNameLen + 2);
    console.log(`  ${chalk.cyan(padded)} ${chalk.dim('—')} ${chalk.white(tutorial.description?.slice(0, 60))}${tutorial.description?.length > 60 ? '...' : ''}`);
    console.log(`  ${chalk.dim(' '.repeat(maxNameLen + 4))} ${chalk.gray(`${steps} steps`)}`);
    console.log('');
  }
}
