import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = join(__dirname, '../..');

let tauriProcess = null;

/**
 * Launch the Tauri application
 * @returns {Promise<ChildProcess>} The spawned process
 */
export async function launchApp() {
    if (tauriProcess) {
        console.log('App is already running');
        return tauriProcess;
    }

    console.log('Launching mclocks application...');
    
    tauriProcess = spawn('pnpm', ['tauri', 'dev'], {
        cwd: projectRoot,
        shell: true,
        stdio: 'pipe'
    });

    tauriProcess.stdout.on('data', (data) => {
        console.log(`[Tauri] ${data.toString()}`);
    });

    tauriProcess.stderr.on('data', (data) => {
        console.error(`[Tauri Error] ${data.toString()}`);
    });

    tauriProcess.on('close', (code) => {
        console.log(`Tauri process exited with code ${code}`);
        tauriProcess = null;
    });

    // Wait a bit for the app to start
    await new Promise(resolve => setTimeout(resolve, 5000));

    return tauriProcess;
}

/**
 * Close the application
 */
export async function closeApp() {
    if (tauriProcess) {
        console.log('Closing mclocks application...');
        tauriProcess.kill();
        tauriProcess = null;
    }
}

