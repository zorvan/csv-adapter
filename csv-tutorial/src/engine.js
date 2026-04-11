const fs = require('fs');
const path = require('path');
const os = require('os');
const prompts = require('prompts');
const chalk = require('chalk');
const { sleep } = require('./utils');

const PROGRESS_DIR = path.join(os.homedir(), '.csv-tutorial', 'progress');

/**
 * TutorialRunner - Manages the execution of a tutorial
 */
class TutorialRunner {
  /**
   * @param {object} renderer - The renderer module
   * @param {object} options - Runner options
   * @param {boolean} [options.interactive=true] - Whether to wait for user input
   */
  constructor(renderer, options = {}) {
    this.renderer = renderer;
    this.interactive = options.interactive !== false;
    this.startTime = Date.now();
    this.currentStep = 0;
  }

  /**
   * Run the entire tutorial
   * @param {object} tutorial - Tutorial definition
   * @returns {Promise<object>} Completion stats
   */
  async run(tutorial) {
    this.startTime = Date.now();

    // Clear screen and show title
    this.renderer.clearScreen();

    const title = await this.renderer.renderHeader(tutorial.title, { font: 'ANSI Shadow', color: 'cyan' });
    console.log(title);

    if (tutorial.description) {
      console.log(chalk.dim('━'.repeat(60)));
      console.log(chalk.whiteBright(tutorial.description));
      console.log(chalk.dim('━'.repeat(60)));
    }

    if (tutorial.prerequisites) {
      console.log(chalk.yellow('\nPrerequisites:'));
      for (const prereq of tutorial.prerequisites) {
        console.log(chalk.dim('  • ') + chalk.white(prereq));
      }
    }

    // Check for saved progress
    const savedProgress = this.loadProgress(tutorial.id);
    if (savedProgress && savedProgress.step > 0) {
      console.log(chalk.yellow(`\n  Resuming from Step ${savedProgress.step}/${tutorial.steps.length}`));
      console.log(chalk.dim('  (Use --restart flag to start from the beginning)\n'));
      this.currentStep = savedProgress.step;
    }

    // Show "press Enter to begin"
    if (this.interactive) {
      await this.waitForUser(chalk.green('\n  Press Enter to begin the tutorial...'));
    }

    // Run each step
    const totalSteps = tutorial.steps.length;
    for (let i = this.currentStep; i < totalSteps; i++) {
      this.currentStep = i;
      const step = tutorial.steps[i];

      await this.runStep(step, i + 1, totalSteps);

      // Save progress after each step
      this.saveProgress(tutorial.id, i + 1);

      // Wait for user before next step (unless it's the last one)
      if (i < totalSteps - 1 && this.interactive) {
        await this.waitForUser(chalk.dim('\n  Press Enter to continue to the next step...'));
      }
    }

    // Generate completion stats
    const stats = {
      tutorialName: tutorial.title,
      completionDate: new Date().toISOString().split('T')[0],
      completionTime: this.formatDuration(Date.now() - this.startTime),
      stepsCompleted: totalSteps,
      totalTimeMs: Date.now() - this.startTime,
      badges: this.generateBadges(totalSteps, Date.now() - this.startTime),
      certificateId: 'CSV-' + this.randomHex(4).toUpperCase()
    };

    return stats;
  }

  /**
   * Run a single step
   * @param {object} step - Step definition
   * @param {number} current - Current step number (1-indexed)
   * @param {number} total - Total steps
   */
  async runStep(step, current, total) {
    // Render step header
    console.log(this.renderer.renderStep(step, current, total));

    // Show code example if provided
    if (step.code) {
      console.log(this.renderer.renderCode(step.code.code, step.code.language || 'javascript'));
      console.log('');
    }

    // Show pro tip if provided
    if (step.proTip) {
      console.log(this.renderer.renderProTip(step.proTip));
    }

    // Execute the action
    if (step.action) {
      const result = await step.action();

      // Show result
      if (step.result) {
        if (typeof step.result === 'function') {
          console.log(step.result(result));
        } else if (typeof step.result === 'string') {
          console.log(this.renderer.renderResult(step.result));
        } else if (typeof step.result === 'object') {
          // Merge action result with static result
          const merged = { ...result, ...step.result };
          console.log(this.renderer.renderResult(merged));
        }
      } else if (result) {
        console.log(this.renderer.renderResult(result));
      }
    }
  }

  /**
   * Wait for user to press Enter
   * @param {string} [message] - Optional prompt message
   */
  async waitForUser(message) {
    const prompt = message || chalk.dim('\n  Press Enter to continue...');
    process.stdout.write(prompt);

    await new Promise(resolve => {
      const onKeyPress = (chunk, key) => {
        if (key && key.name === 'enter') {
          process.stdin.setRawMode(false);
          process.stdin.resume();
          process.stdin.removeListener('keypress', onKeyPress);
          console.log();
          resolve();
        }
      };

      process.stdin.setRawMode(true);
      process.stdin.resume();
      process.stdin.on('keypress', onKeyPress);
    });
  }

  /**
   * Save progress to file
   * @param {string} tutorialId - Tutorial identifier
   * @param {number} step - Current step number
   */
  saveProgress(tutorialId, step) {
    try {
      if (!fs.existsSync(PROGRESS_DIR)) {
        fs.mkdirSync(PROGRESS_DIR, { recursive: true });
      }
      const progressPath = path.join(PROGRESS_DIR, `${tutorialId}.json`);
      fs.writeFileSync(progressPath, JSON.stringify({
        tutorialId,
        step,
        timestamp: Date.now()
      }), 'utf-8');
    } catch (err) {
      // Silently ignore progress save errors
    }
  }

  /**
   * Load saved progress
   * @param {string} tutorialId
   * @returns {object|null}
   */
  loadProgress(tutorialId) {
    try {
      const progressPath = path.join(PROGRESS_DIR, `${tutorialId}.json`);
      if (fs.existsSync(progressPath)) {
        const data = JSON.parse(fs.readFileSync(progressPath, 'utf-8'));
        // Only return progress if it's less than 24 hours old
        if (Date.now() - data.timestamp < 24 * 60 * 60 * 1000) {
          return data;
        }
      }
    } catch (err) {
      // Silently ignore progress load errors
    }
    return null;
  }

  /**
   * Format a duration in milliseconds to human-readable string
   * @param {number} ms
   * @returns {string}
   */
  formatDuration(ms) {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  }

  /**
   * Generate badges based on performance
   * @param {number} stepsCompleted
   * @param {number} durationMs
   * @returns {string[]}
   */
  generateBadges(stepsCompleted, durationMs) {
    const badges = [];
    const minutes = durationMs / 60000;

    if (minutes < 3) badges.push('Speed Demon');
    else if (minutes < 5) badges.push('Fast Learner');
    else if (minutes < 10) badges.push('Methodical Thinker');

    if (stepsCompleted >= 7) badges.push('Cross-Chain Certified');
    if (stepsCompleted >= 10) badges.push('Tutorial Master');

    badges.push('CSV Explorer');

    return badges;
  }

  /**
   * Generate a random hex string
   * @param {number} bytes
   * @returns {string}
   */
  randomHex(bytes) {
    const crypto = require('crypto');
    return crypto.randomBytes(bytes).toString('hex');
  }
}

module.exports = { TutorialRunner };
