export interface Device {
  id: string;
  nickname: string;
  isOnline: boolean;
  isMyDevice?: boolean;
  lastSeen?: Date;
}

export interface DirectoryContents {
  folders: string[];
  files: string[];
}

export interface FolderNode {
  name: string;
  children?: FolderNode[];
  isFile?: boolean;
}
