import { rgb24 } from 'jsr:@gnome/ansi@0.2';

class Check {
    async exists(path: string): Promise<boolean> {
        try {
            await Deno.stat(path);
        } catch (e) {
            if (e instanceof Deno.errors.NotFound) {
                return false;
            }
        }
        return true;
    }
}

export const check = new Check();

class Shell {
    args(): string[] {
        return Deno.args;
    }

    async spawn(command: string, args: string[]): Promise<boolean> {
        const cmd = new Deno.Command(command, {
            args,
        });
        const proc = cmd.spawn();
        const output = await proc.output();
        return output.success;
    }

    _gitRootDirectory: string | null = null;
    async gitRootDirectory(): Promise<string> {
        if (!this._gitRootDirectory) {
            const { stdout } = await new Deno.Command('git', {
                args: ['rev-parse', '--show-toplevel'],
                stdout: 'piped',
            }).output();
            this._gitRootDirectory = new TextDecoder().decode(stdout).trim();
        }
        return this._gitRootDirectory;
    }

    async gitRelativeDirectory(): Promise<string> {
        const gitRoot = await this.gitRootDirectory();
        const currentDir = Deno.cwd();
        const relativeDir = currentDir.replace(`${gitRoot}/`, '');
        return relativeDir;
    }
}

export const shell = new Shell();

class Ensure {
    string(str: unknown): string {
        if (typeof str !== 'string') {
            console.error(`ensure failed: ${str} is not a string`);
            Deno.exit(1);
        }
        return str;
    }

    async isDirectory(path: string): Promise<string> {
        const info = await Deno.stat(path);
        if (!info.isDirectory) {
            console.error(`ensure failed: ${path} is not a directory`);
            Deno.exit(1);
        }
        return path;
    }

    async ls(dir: string, { extensions }: { extensions?: string[] } = {}): Promise<string[]> {
        let files = await Array.fromAsync(await Deno.readDir(dir));
        if (extensions) {
            files = files.filter((f) => extensions.includes(f.name.split('.').pop() ?? ''));
        }
        return files.map((f) => `${dir}/${f.name}`);
    }

    async spawn(command: string, args: string[]) {
        if (!(await shell.spawn(command, args))) {
            console.error(`ensure failed: ${command} ${args.join(' ')} failed`);
            Deno.exit(1);
        }
    }

    async fileContents(path: string, content: () => Promise<string> | string) {
        // 📐 By design, do nothing if the file is already there
        if (await check.exists(path)) {
            return;
        }

        // Ensure directory of path exists
        const dir = path.split('/').slice(0, -1).join('/');
        await Deno.mkdir(dir, { recursive: true });

        // Write content to file
        const text = typeof content === 'function' ? await content() : content;
        await Deno.writeTextFile(path, text);
    }
}

export const ensure = new Ensure();

class Print {
    banner(msg: string) {
        console.log(rgb24(`*** ${msg}`, { r: 255, g: 100, b: 10 }));
    }
}

export const print = new Print();

class Util {
    dateYYYYMMDD(separator: '-' | '.' | '/' | '', date: Date = new Date()): string {
        const base = date.toISOString().split('T')[0];
        return base.replace(/-/g, separator);
    }
}

export const util = new Util();