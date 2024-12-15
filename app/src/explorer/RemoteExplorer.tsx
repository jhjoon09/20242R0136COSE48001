import {
  VscChevronDown,
  VscChevronRight,
  VscFile,
  VscFolder,
  VscFolderOpened,
} from 'react-icons/vsc';
import React, { useState, useEffect } from 'react';
import './Explorer.css';
import { invoke } from '@tauri-apps/api/core';
import { FolderNode } from './Types';

interface RemoteExplorerProps {
  curDeviceName: string;
  deviceId: string;
  folderMap: string[];
  deviceName: string;
}

interface FolderItemProps {
  node: FolderNode;
  path: string;
  depth: number;
  onFileSelect?: (path: string | null) => void;
  onFolderSelect?: (path: string) => void;
  selectedPath: string | null;
}

const FolderItem: React.FC<FolderItemProps> = ({
  node,
  path,
  depth,
  onFileSelect,
  onFolderSelect,
  selectedPath,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const isSelected = node.isFile && selectedPath === path;

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (node.isFile) {
      if (isSelected) {
        onFileSelect?.(null);
      } else {
        onFileSelect?.(path);
      }
    }
  };

  const handleToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!node.isFile) {
      setIsOpen(!isOpen);
      if (isOpen && selectedPath?.startsWith(path + '/')) {
        onFileSelect?.(null);
      }
    }
  };

  return (
    <div>
      <div
        className={`list-item flex items-center gap-2 cursor-pointer p-2
          ${
            isSelected
              ? 'bg-blue-100 dark:bg-gray-800'
              : 'hover:bg-gray-200 dark:hover:bg-gray-700'
          }
          `}
        style={{
          paddingLeft: `${depth * 1.5}rem`,
          backgroundColor:
            isSelected &&
            window.matchMedia('(prefers-color-scheme: dark)').matches
              ? '#4B5563'
              : undefined,
          color:
            isSelected &&
            window.matchMedia('(prefers-color-scheme: dark)').matches
              ? 'white'
              : undefined,
        }}
        onClick={node.isFile ? handleClick : handleToggle}
      >
        {!node.isFile && (
          <span onClick={handleToggle}>
            {isOpen ? (
              <VscChevronDown className="w-4 h-4" />
            ) : (
              <VscChevronRight className="w-4 h-4" />
            )}
          </span>
        )}
        {node.isFile ? (
          <VscFile className="w-4 h-4 text-gray-500 dark:text-gray-300" />
        ) : isOpen ? (
          <VscFolderOpened className="w-4 h-4 text-gray-500" />
        ) : (
          <VscFolder className="w-4 h-4 text-gray-500" />
        )}
        <span className="flex-1">{node.name}</span>
      </div>

      {isOpen && node.children && (
        <div>
          {node.children.map((child, idx) => (
            <FolderItem
              key={`${path}-${idx}`}
              node={child}
              path={`${path}/${child.name}`}
              depth={depth + 1}
              onFileSelect={onFileSelect}
              onFolderSelect={onFolderSelect}
              selectedPath={selectedPath}
            />
          ))}
        </div>
      )}
    </div>
  );
};

interface DownloadModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  savePath: string;
  saveFileName: string;
  onFileNameChange: (name: string) => void;
  fileNameError: string;
  deviceName: string;
}

const DownloadModal: React.FC<DownloadModalProps> = ({
  isOpen,
  onClose,
  onConfirm,
  savePath,
  saveFileName,
  onFileNameChange,
  fileNameError,
  deviceName,
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-xl min-w-[400px]">
        <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100">
          {deviceName}으로 다운로드
        </h3>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              저장 경로 (Workspace)
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={savePath}
                readOnly
                className="flex-1 px-3 py-2 border rounded-lg bg-gray-50 text-gray-700
                  dark:bg-gray-700 dark:border-gray-600 dark:text-gray-300 cursor-not-allowed"
                placeholder="다운로드할 상위 폴더 경로"
              />
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              파일 이름
            </label>
            <input
              type="text"
              value={saveFileName}
              onChange={(e) => onFileNameChange(e.target.value)}
              className={`w-full px-3 py-2 border rounded-lg focus:outline-none 
                focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600
                dark:text-gray-300 dark:placeholder-gray-500
                ${
                  fileNameError
                    ? 'border-red-500 focus:ring-red-500'
                    : 'border-gray-300 dark:border-gray-600'
                }`}
              placeholder="저장할 파일 이름을 입력하세요"
            />
            {fileNameError && (
              <p className="mt-1 text-sm text-red-500 dark:text-red-400">
                {fileNameError}
              </p>
            )}
          </div>
        </div>

        <div className="flex justify-end gap-2 mt-6">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-600 hover:text-gray-700 
              dark:text-gray-400 dark:hover:text-gray-300 transition-colors"
          >
            취소
          </button>
          <button
            onClick={onConfirm}
            disabled={!savePath || !saveFileName}
            className="px-4 py-2 bg-blue-500 text-white rounded 
              hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700
              disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            확인
          </button>
        </div>
      </div>
    </div>
  );
};

const SuccessCheck: React.FC<{ show: boolean }> = ({ show }) => {
  if (!show) return null;

  return (
    <div className="fixed inset-0 flex items-center justify-center pointer-events-none z-50">
      <div className="animate-success-check bg-green-100 dark:bg-green-900 rounded-full p-6">
        <svg
          className="w-24 h-24 text-green-500 dark:text-green-300"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={3}
            d="M5 13l4 4L19 7"
          />
        </svg>
      </div>
    </div>
  );
};

const RemoteExplorer: React.FC<RemoteExplorerProps> = ({
  curDeviceName,
  deviceId,
  folderMap,
  deviceName,
}) => {
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [showModal, setShowModal] = useState(false);
  const [savePath, setSavePath] = useState<string>('');
  const [saveFileName, setSaveFileName] = useState<string>('');
  const [fileNameError, setFileNameError] = useState<string>('');
  const [showSuccess, setShowSuccess] = useState(false);

  useEffect(() => {
    const getWorkspace = async () => {
      const ws = await invoke('get_workspace');
      setSavePath(ws as string);
    };
    getWorkspace();
  }, []);

  useEffect(() => {
    if (selectedPath) {
      setSaveFileName(selectedPath.split('/').pop() || '');
    }
  }, [selectedPath]);

  const handleFileSelect = (path: string | null) => {
    if (!path) {
      setSelectedPath(null);
      return;
    }
    setSelectedPath(path);
  };

  const handleDownload = async () => {
    if (!selectedPath) return;
    setShowModal(true);
  };

  const checkFileExists = async (fileName: string): Promise<boolean> => {
    try {
      const data = await invoke<{ files: string[]; folders: string[] }>(
        'get_files',
        {
          path: savePath,
        },
      );
      return data.files.includes(fileName);
    } catch (error) {
      console.error('Error checking file:', error);
      return false;
    }
  };

  const confirmDownload = async () => {
    if (!savePath || !saveFileName) return;

    const exists = await checkFileExists(saveFileName);

    if (exists) {
      setFileNameError('이미 같은 이름의 파일이 존재합니다');
      return;
    }
    console.log('File name verified');

    try {
      await invoke('recive_file', {
        id: deviceId,
        source: '.' + selectedPath?.substring(4),
        target: `./${saveFileName}`,
      });
      setShowModal(false);
      setSelectedPath(null);
      setSavePath('');
      setSaveFileName('');
      setFileNameError('');

      setShowSuccess(true);
      setTimeout(() => {
        setShowSuccess(false);
      }, 1500);
    } catch (error) {
      alert('파일 다운로드 중 에러 발생: ' + error);
      console.error('Error downloading file:', error);
    }
  };

  const buildFileAndFolderTree = (paths: string[]): FolderNode[] => {
    const tree: { [key: string]: FolderNode } = {};
    const root: FolderNode[] = [];

    paths.forEach((path) => {
      const parts = path.split('/').filter(Boolean);
      let currentPath = '';

      parts.forEach((part, index) => {
        currentPath = currentPath ? `${currentPath}/${part}` : part;

        let node = tree[currentPath];
        if (!node) {
          const isFile = index === parts.length - 1 && !path.endsWith('/');
          node = {
            name: part,
            isFile,
            children: isFile ? undefined : [],
          };
          tree[currentPath] = node;

          if (index === 0) {
            root.push(node);
          } else {
            const parentPath = currentPath.split('/').slice(0, -1).join('/');
            const parent = tree[parentPath];
            if (parent && parent.children) {
              parent.children.push(node);
            }
          }
        }
        // if (!isFile && node.children) {
        //   currentLevel = node.children;
        // }
      });
    });

    return root;
  };

  const items = buildFileAndFolderTree(folderMap);

  const handleFileNameChange = (name: string) => {
    setSaveFileName(name);
    setFileNameError('');
  };

  const handleCloseModal = () => {
    setShowModal(false);
    setFileNameError('');
  };

  return (
    <div className="file-explorer relative h-full">
      <div className="file-explorer-header">
        <h2 className="text-lg font-semibold text-gray-700 dark:text-gray-200">
          {deviceName}의 파일
        </h2>
      </div>
      <div className="file-list">
        {items.map((node, idx) => (
          <FolderItem
            key={idx}
            node={node}
            path={node.name}
            depth={0}
            onFileSelect={handleFileSelect}
            selectedPath={selectedPath}
          />
        ))}
      </div>

      {selectedPath && (
        <button
          onClick={handleDownload}
          className="fixed bottom-8 right-8 bg-blue-500 hover:bg-blue-600 
            text-white px-6 py-3 rounded-full shadow-lg text-lg font-medium
            transition-all duration-300 ease-out transform
            hover:scale-105 hover:shadow-xl
            animate-fade-scale-up flex items-center gap-3"
        >
          <svg
            className="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
            />
          </svg>
          <span>다운로드</span>
        </button>
      )}

      <DownloadModal
        isOpen={showModal}
        onClose={handleCloseModal}
        onConfirm={confirmDownload}
        savePath={savePath}
        saveFileName={saveFileName}
        onFileNameChange={handleFileNameChange}
        fileNameError={fileNameError}
        deviceName={curDeviceName}
      />

      <SuccessCheck show={showSuccess} />
    </div>
  );
};

export default RemoteExplorer;
