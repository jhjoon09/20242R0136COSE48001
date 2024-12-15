import { BsLaptop, BsPhone } from 'react-icons/bs';
import { FaAndroid, FaApple, FaLinux, FaWindows } from 'react-icons/fa';

export interface Device {
  id: string;
  nickname: string;
  os: string;
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

export const getOsIcon = (os: string) => {
  const osLower = os.toLowerCase();

  if (osLower.includes('android')) {
    return <FaAndroid className="w-5 h-5 text-green-500" />;
  }
  if (
    osLower.includes('ios') ||
    osLower.includes('iphone') ||
    osLower.includes('ipad')
  ) {
    return <FaApple className="w-5 h-5 text-gray-600 dark:text-gray-300" />;
  }
  if (osLower.includes('windows')) {
    return <FaWindows className="w-5 h-5 text-blue-500" />;
  }
  if (osLower.includes('linux') || osLower.includes('ubuntu')) {
    return <FaLinux className="w-5 h-5 text-orange-500" />;
  }
  if (osLower.includes('mac')) {
    return <FaApple className="w-5 h-5 text-gray-600 dark:text-gray-300" />;
  }
  // 기본 아이콘
  return osLower.includes('mobile') ? (
    <BsPhone className="w-5 h-5 text-gray-500" />
  ) : (
    <BsLaptop className="w-5 h-5 text-gray-500" />
  );
};
