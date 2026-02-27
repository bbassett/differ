export type RefInfo = { name: string; refType: string };

export type DiffLine = {
  lineType: string;
  content: string;
  oldNum: number | null;
  newNum: number | null;
};

export type DiffHunk = {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
};

export type DiffFile = {
  path: string;
  status: string;
  oldPath: string | null;
  hunks: DiffHunk[];
};

export type DiffResult = {
  baseRef: string;
  compareRef: string;
  files: DiffFile[];
};
