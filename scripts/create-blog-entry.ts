#!/usr/bin/env -S deno run --allow-all
import { ensure, util } from './shell.ts';

// Get date in YYYYMMDD format
const date = util.dateYYYYMMDD('-');
const filename = `./blog/${date}/${date}.md`;
await ensure.fileContents(
    filename,
    () =>
        `
# ${date}


`.trim() + '\n'
);

await ensure.command('code', [filename]);
