import { VscChevronDown, VscChevronRight, VscFile } from 'react-icons/vsc'; // VS Code 스타일
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './Explorer.css';
import { Device, DirectoryContents } from './Types';
import NavigationButtons from './components/NavigationButtons';

interface FolderItemProps {
  name: string;
  path: string;
  depth: number;
  onFileSelect?: (path: string | null) => void;
  selectedFile: string | null;
}

const FolderItem: React.FC<FolderItemProps> = ({
  name,
  path,
  depth,
  onFileSelect,
  selectedFile,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [subItems, setSubItems] = useState<DirectoryContents>({
    folders: [],
    files: [],
  });

  const fetchSubItems = async () => {
    try {
      const data: DirectoryContents = await invoke('get_files', { path });
      setSubItems(data);
    } catch (error) {
      console.error('Error fetching subfolder:', error);
    }
  };

  const handleToggle = async () => {
    if (!isOpen) {
      await fetchSubItems();
    }
    setIsOpen(!isOpen);
  };

  return (
    <div>
      <div
        className="list-item flex items-center hover:bg-gray-200 cursor-pointer gap-2 
          dark:hover:bg-gray-700 p-2"
        style={{ paddingLeft: `${depth * 1.5}rem` }}
        onClick={handleToggle}
      >
        {isOpen ? (
          <VscChevronDown className="w-4 h-4" />
        ) : (
          <VscChevronRight className="w-4 h-4" />
        )}
        <span className="ml-2">{name}</span>
      </div>

      {isOpen && (
        <div>
          {subItems.folders.map((folder, idx) => (
            <FolderItem
              key={`${path}-folder-${idx}`}
              name={folder}
              path={`${path}/${folder}`}
              depth={depth + 1}
              onFileSelect={onFileSelect}
              selectedFile={selectedFile}
            />
          ))}
          {subItems.files.map((file, idx) => {
            const filePath = `${path}/${file}`;
            const isSelected = selectedFile === filePath;

            return (
              <div
                key={`${path}-file-${idx}`}
                className={`list-item flex items-center cursor-pointer p-2
                  ${
                    isSelected
                      ? 'bg-blue-100 dark:bg-gray-800'
                      : 'hover:bg-gray-200 dark:hover:bg-gray-700'
                  }`}
                style={{
                  paddingLeft: `${(depth + 1) * 1.5}rem`,
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
                onClick={() => {
                  const filePath = `${path}/${file}`;
                  onFileSelect?.(filePath);
                }}
              >
                <VscFile className="w-4 h-4" />
                <span className="ml-2">{file}</span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

interface FileExplorerProps {
  onFileSelect?: (filePath: string | null) => void;
  devices: Device[];
  myDeviceId: string | null;
}

interface UploadModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (deviceId: string, fileName: string) => void;
  devices: Device[];
  selectedFile: string | null;
  myDeviceId: string | null;
}

const UploadModal: React.FC<UploadModalProps> = ({
  isOpen,
  onClose,
  onConfirm,
  devices,
  selectedFile,
  myDeviceId,
}) => {
  const [selectedDevice, setSelectedDevice] = useState<string>('');
  const [fileName, setFileName] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    if (selectedFile) {
      setFileName(selectedFile.split('/').pop() || '');
    }
  }, [selectedFile]);

  if (!isOpen) return null;

  const handleConfirm = () => {
    if (!selectedDevice) {
      setError('기기를 선택해주세요');
      return;
    }
    if (!fileName) {
      setError('파일 이름을 입력해주세요');
      return;
    }
    onConfirm(selectedDevice, fileName);
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-[1001]">
      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-xl min-w-[400px]">
        <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100">
          파일 업로드
        </h3>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              대상 기기
            </label>
            <select
              value={selectedDevice}
              onChange={(e) => {
                setSelectedDevice(e.target.value);
                setError('');
              }}
              className="w-full px-3 py-2 border rounded-lg focus:outline-none 
                focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600
                dark:text-gray-300"
            >
              <option value="">기기 선택</option>
              {devices
                .filter((device) => device.id !== myDeviceId && device.isOnline)
                .map((device) => (
                  <option key={device.id} value={device.id}>
                    {device.nickname}
                  </option>
                ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              파일 이름
            </label>
            <input
              type="text"
              value={fileName}
              onChange={(e) => {
                setFileName(e.target.value);
                setError('');
              }}
              className="w-full px-3 py-2 border rounded-lg focus:outline-none 
                focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600
                dark:text-gray-300"
              placeholder="저장할 파일 이름을 입력하세요"
            />
          </div>

          {error && (
            <p className="text-sm text-red-500 dark:text-red-400">{error}</p>
          )}
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
            onClick={handleConfirm}
            className="px-4 py-2 bg-[#862633] hover:bg-[#a62f3f]  text-white rounded 
              dark:bg-[#862633] dark:hover:bg-[#a62f3f] 
              transition-colors"
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

const FileExplorer: React.FC<FileExplorerProps> = ({
  onFileSelect,
  devices,
  myDeviceId,
}) => {
  const [currentPath, setCurrentPath] = useState<string>('~');
  const [folders, setFolders] = useState<string[]>([]);
  const [files, setFiles] = useState<string[]>([]);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [showUploadModal, setShowUploadModal] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);
  const [isPathExpanded, setIsPathExpanded] = useState(false);
  const [isExiting, setIsExiting] = useState(false);

  const fetchFiles = async (path: string) => {
    try {
      const data: DirectoryContents = await invoke('get_files', { path });
      setCurrentPath(path);
      setFiles(data.files);
      setFolders(data.folders);
      setSelectedFile(null);
    } catch (error) {
      console.error('Error fetching files:', error);
    }
  };

  useEffect(() => {
    const initWorkspace = async () => {
      try {
        const workspace = await invoke<string>('get_workspace');
        setCurrentPath(workspace);
        await fetchFiles(workspace);
      } catch (error) {
        console.error('Error fetching workspace:', error);
        await fetchFiles('~');
      }
    };

    initWorkspace();
  }, []);

  const goToWorkspace = async () => {
    try {
      const workspace = await invoke<string>('get_workspace');
      await fetchFiles(workspace);
    } catch (error) {
      console.error('Error navigating to workspace:', error);
    }
  };

  const handleUpload = async (deviceId: string, fileName: string) => {
    if (!selectedFile) return;

    try {
      await invoke('send_file', {
        id: deviceId,
        source: selectedFile,
        target: `./${fileName}`,
      });
      setShowUploadModal(false);
      setSelectedFile(null);

      setShowSuccess(true);
      setTimeout(() => {
        setShowSuccess(false);
      }, 1500);
    } catch (error) {
      console.error('Error uploading file:', error);
      alert('파일 업로드 중 에러가 발생했습니다: ' + error);
    }
  };

  const truncatePath = (path: string) => {
    if (!isPathExpanded && path.length > 20) {
      return path.substring(0, 15) + '...' + path.substring(path.length - 5);
    }
    return path;
  };

  const openUploadModal = () => {
    setShowUploadModal(true);
  };

  const handleFileDeselect = () => {
    setIsExiting(true);
    setTimeout(() => {
      setSelectedFile(null);
      setIsExiting(false);
    }, 300);
  };

  return (
    <div className="file-explorer relative h-full">
      <div className="file-explorer-header">
        <div className="flex-1 min-w-0">
          <h2 className="text-lg font-semibold dark:text-white">파일 탐색기</h2>
          <div className="flex flex-col gap-1">
            <div className="flex items-center gap-2">
              <p className="text-sm text-gray-600 dark:text-gray-400">
                {isPathExpanded ? (
                  <span className="break-all">{currentPath}</span>
                ) : (
                  <span className="truncate">{truncatePath(currentPath)}</span>
                )}
              </p>
              {currentPath.length > 20 && (
                <button
                  onClick={() => setIsPathExpanded(!isPathExpanded)}
                  className="text-xs text-[#862633] hover:text-[#a62f3f] underline shrink-0 
                    transition-colors inline-flex items-center gap-1 self-start"
                >
                  {isPathExpanded ? '접기' : '더보기'}
                  <svg
                    className={`w-4 h-4 transition-transform ${
                      isPathExpanded ? 'rotate-180' : ''
                    }`}
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M19 9l-7 7-7-7"
                    />
                  </svg>
                </button>
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="file-list">
        {folders.map((folder, idx) => (
          <FolderItem
            key={`folder-${idx}`}
            name={folder}
            path={`${currentPath}/${folder}`}
            depth={0}
            onFileSelect={(path: string | null) => {
              setSelectedFile(path);
              onFileSelect?.(path);
            }}
            selectedFile={selectedFile}
          />
        ))}

        {files.map((file, idx) => {
          const filePath = `${currentPath}/${file}`;
          const isSelected = selectedFile === filePath;

          return (
            <div
              key={`file-${idx}`}
              className={`list-item flex items-center cursor-pointer gap-2 p-2
                ${
                  isSelected
                    ? 'bg-blue-100 dark:bg-gray-800'
                    : 'hover:bg-gray-200 dark:hover:bg-gray-700'
                }`}
              style={{
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
              onClick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                if (selectedFile === filePath) {
                  handleFileDeselect();
                  onFileSelect?.(null);
                } else {
                  setSelectedFile(filePath);
                  onFileSelect?.(filePath);
                }
              }}
            >
              <VscFile className="w-4 h-4 flex-shrink-0" />
              <span>{file}</span>
            </div>
          );
        })}
      </div>

      <NavigationButtons
        onWorkspaceClick={goToWorkspace}
        onUpClick={() =>
          fetchFiles(currentPath.split('/').slice(0, -1).join('/') || '~')
        }
        isUpDisabled={currentPath === '~'}
        showUploadButton={!!selectedFile}
      />

      {selectedFile && (
        <button
          onClick={openUploadModal}
          className={`fixed bottom-8 right-8 bg-[#862633] hover:bg-[#a62f3f] 
            text-white px-6 py-3 rounded-full shadow-lg text-lg font-medium
            transition-all duration-300 ease-out transform
            hover:scale-105 hover:shadow-xl flex items-center gap-3
            ${isExiting ? 'animate-slide-down' : 'animate-slide-up'}`}
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
              d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0l-4 4m4-4v12"
            />
          </svg>
          <span>업로드</span>
        </button>
      )}

      <UploadModal
        isOpen={showUploadModal}
        onClose={() => setShowUploadModal(false)}
        onConfirm={handleUpload}
        devices={devices}
        selectedFile={selectedFile}
        myDeviceId={myDeviceId}
      />

      <SuccessCheck show={showSuccess} />
    </div>
  );
};
export default FileExplorer;
