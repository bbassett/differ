import type { DiffFile } from './types';

let viewed: Record<string, string> = $state({});

function hashFile(file: DiffFile): string {
  let hash = 5381;
  for (const hunk of file.hunks) {
    for (const line of hunk.lines) {
      const str = line.lineType + line.content;
      for (let i = 0; i < str.length; i++) {
        hash = ((hash << 5) + hash + str.charCodeAt(i)) | 0;
      }
    }
  }
  return hash.toString(36);
}

export function toggleViewed(file: DiffFile) {
  const hash = hashFile(file);
  if (file.path in viewed && viewed[file.path] === hash) {
    delete viewed[file.path];
  } else {
    viewed[file.path] = hash;
  }
}

export function isViewed(file: DiffFile): boolean {
  return viewed[file.path] === hashFile(file);
}

export function reconcile(files: DiffFile[]) {
  const paths = new Set(files.map((f) => f.path));
  for (const path of Object.keys(viewed)) {
    if (!paths.has(path)) {
      delete viewed[path];
    }
  }
  for (const file of files) {
    if (file.path in viewed && viewed[file.path] !== hashFile(file)) {
      delete viewed[file.path];
    }
  }
}

export function viewedCount(files: DiffFile[]): { viewed: number; total: number } {
  let count = 0;
  for (const file of files) {
    if (isViewed(file)) count++;
  }
  return { viewed: count, total: files.length };
}
