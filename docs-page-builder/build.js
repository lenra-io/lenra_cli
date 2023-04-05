import * as fs from 'fs/promises';
import Showdown from 'showdown';
import fm from 'front-matter';
import path from 'path';

const converter = new Showdown.Converter({ tables: true });
converter.addExtension({
    type: 'output',
    filter: function (text) {
        const replaced = text
            // Replace markdown links to html
            .replace(/href="([^"]+)"/g, (_, href) => `href="${href.replace(/\.md(#|$)/, '.html$1')}"`);
        return replaced;
    }
});

const githubBasPath = "https://github.com/lenra-io/lenra_cli/blob/beta/";

const docsDir = path.join("..", "docs");
const outDir = "build";

async function buildFiles(src, dest) {
    const files = await fs.readdir(src, { withFileTypes: true });
    const promises = files.map(info => {
        const filePath = path.join(src, info.name);
        if (info.isFile()) return buildFile(filePath, info.name, dest);
        if (!info.isDirectory()) throw new Error("Error building", filePath);
        return buildFiles(filePath, path.join(dest, info.name));
    });
    return Promise.all(promises);
}

/**
 * Build the given file to the target directory
 * @param {string} src 
 * @param {string} filename 
 * @param {string} destDir 
 */
async function buildFile(src, filename, destDir) {
    await fs.mkdir(destDir, { recursive: true });
    if (filename.endsWith(".md")) {
        console.log(`buiding ${src} to ${destDir}`);
        const fmResult = fm(await fs.readFile(src, 'utf8'));
        const baseName = filename.replace(/.md$/, "");
        const title = fmResult.attributes.title ?? (filename === "index.md" ? undefined : baseName);
        const destFile = path.join(destDir, `${baseName}.html`);
        const destJsonFile = destFile + '.json';
        return Promise.all([
            fs.writeFile(destFile, converter.makeHtml(fmResult.body), 'utf8'),
            fs.writeFile(destJsonFile, JSON.stringify({
                ...fmResult.attributes,
                title,
                sourceFile: githubBasPath + src.replace(/^..\//, '')
            }), 'utf8')
        ]);
    }
    else {
        return fs.copyFile(src, path.join(destDir, filename));
    }
}

console.log("Start building doc pages");
buildFiles(docsDir, outDir).then(_ => console.log("Doc pages built"));
