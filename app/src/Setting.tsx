import React, { useState, useEffect } from 'react';
import './Settings.css'; // CSS 파일을 import
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';

interface DirectoryContents {
  folders: string[];
}

const Settings: React.FC = () => {
  const [isFirst, setIsFirst] = useState<boolean>(true);
  const [nickname, setNickname] = useState<string>('');
  const [workspace, setWorkspace] = useState<string>('~');
  const [group, setGroup] = useState<string>('');
  const [domain, setDomain] = useState<string>('');
  const [hash, setHash] = useState<string>('');
  const [serverPort, setServerPort] = useState<string>('');
  const [p2pPort, setP2pPort] = useState<string>('');

  const [openWorkspace, setOpenWorkspace] = useState<boolean>(false);
  const [folders, setFolders] = useState<string[]>([]);
  const [currentPath, setCurrentPath] = useState<string>('~');
  const navigate = useNavigate();

  const [showOptional, setShowOptional] = useState<boolean>(false);

  const [isExpanded, setIsExpanded] = useState<boolean>(false);

  useEffect(() => {
    const checkFirst = async () => {
      const isFirstRun = await invoke<boolean>('is_first_run');
      setIsFirst(isFirstRun);

      // 처음이 아닌 경우 기존 설정 불러오기
      if (!isFirstRun) {
        await invoke('load_config');
        const nickname = await invoke<string>('get_nickname');
        const workspace = await invoke<string>('get_workspace');

        setNickname(nickname);
        setWorkspace(workspace);

        // config.yaml에서 추가 설정값들을 불러오는 함수 필요
        try {
          const config = await invoke<{
            domain: string;
            hash: string;
            server_port: number;
            p2p_port: number;
          }>('get_current_config');

          setDomain(config.domain);
          setHash(config.hash);
          setServerPort(config.server_port.toString());
          setP2pPort(config.p2p_port.toString());
        } catch (error) {
          console.error('설정값을 불러오는데 실패했습니다:', error);
        }
      }
    };
    checkFirst();
  }, []);

  // 디렉토리 목록 가져오기
  const fetchFolders = async (path: string) => {
    try {
      const data: DirectoryContents = await invoke('get_files', { path });
      setCurrentPath(path);
      setFolders(data.folders);
    } catch (error) {
      console.error('Error fetching folders:', error);
    }
  };

  // 초기 로드
  const handleOpenWorkspace = () => {
    setOpenWorkspace(true);
    fetchFolders('~');
  };

  // 상위 폴더 이동
  const handleGoUp = () => {
    if (currentPath === '~') {
      return;
    }
    const parentPath = currentPath.split('/').slice(0, -1).join('/') || '~';
    fetchFolders(parentPath);
  };

  // 폴더 클릭 처리
  const handleFolderClick = (folderName: string) => {
    const newPath = `${currentPath}/${folderName}`;
    fetchFolders(newPath);
  };

  // 폴더 선택 처리
  const handleFolderSelect = (folderName: string) => {
    const selectedPath = `${currentPath}/${folderName}`;
    setWorkspace(selectedPath);
    setOpenWorkspace(false);
  };

  const goMain = async () => {
    navigate('/', { replace: true });
  };

  // 설정 저장
  const saveSetting = async () => {
    try {
      await invoke('init_config', {
        workspace,
        group,
        nickname,
        domain: domain.trim() || null,
        hash: hash.trim() || null,
        server_port: serverPort.trim() ? parseInt(serverPort) : null,
        p2p_port: p2pPort.trim() ? parseInt(p2pPort) : null,
      });
      navigate('/', { replace: true });
      if (!isFirst) {
        alert('설정을 적용하기 위해서는 재시작이 필요합니다!');
      }
    } catch (error) {
      console.error('Error saving:', error);
      alert('Failed to save. Try again.');
    }
  };

  const truncateWorkspace = (path: string) => {
    if (path.length > 30 && !isExpanded) {
      return path.substring(0, 30) + '...';
    }
    return path;
  };

  if (openWorkspace) {
    return (
      <div className="file-explorer relative h-full bg-white dark:bg-gray-900">
        <div className="file-explorer-header border-b border-gray-200 dark:border-gray-700">
          <h2 className="text-lg font-semibold text-gray-700 dark:text-gray-200">
            작업 공간 선택
          </h2>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            현재 경로: {currentPath}
          </p>
        </div>

        <div className="p-4 space-y-4">
          <button
            onClick={handleGoUp}
            disabled={currentPath === '~/'}
            className={`px-4 py-2 rounded-lg transition-colors flex items-center gap-2
              ${
                currentPath === '~/'
                  ? 'bg-gray-100 text-gray-400 cursor-not-allowed dark:bg-gray-800 dark:text-gray-600'
                  : 'bg-[#862633] hover:bg-[#a62f3f] text-white'
              }`}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="h-5 w-5"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path
                fillRule="evenodd"
                d="M14.707 12.707a1 1 0 01-1.414 0L10 9.414l-3.293 3.293a1 1 0 01-1.414-1.414l4-4a1 1 0 011.414 0l4 4a1 1 0 010 1.414z"
                clipRule="evenodd"
              />
            </svg>
            상위 폴더로
          </button>

          <div className="space-y-2">
            {folders.map((folder, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 rounded-lg
                  bg-gray-50 dark:bg-gray-800 hover:bg-gray-100 
                  dark:hover:bg-gray-700 transition-colors"
              >
                <div
                  className="flex items-center gap-2 flex-1 cursor-pointer"
                  onClick={() => handleFolderClick(folder)}
                >
                  <span className="text-gray-500 dark:text-gray-400">📁</span>
                  <span className="text-gray-700 dark:text-gray-200">
                    {folder}
                  </span>
                </div>
                <button
                  onClick={() => handleFolderSelect(folder)}
                  className="px-4 py-2 bg-[#862633] hover:bg-[#a62f3f] text-white rounded-lg
                    transition-colors"
                >
                  선택
                </button>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="file-explorer relative h-full bg-white dark:bg-gray-900">
      <div className="file-explorer-header border-b border-gray-200 dark:border-gray-700 p-6">
        <div className="flex items-center gap-3">
          <svg
            className="w-8 h-8 text-gray-800 dark:text-gray-100 flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
            />
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
          <h1 className="text-3xl font-bold text-gray-800 dark:text-gray-100 leading-none translate-y-[6px]">
            설정
          </h1>
        </div>
      </div>

      <div className="p-6 space-y-6 bg-white dark:bg-gray-900">
        {/* 작업 공간 설정 */}
        <div className="space-y-2">
          <label className="block text-m font-medium text-gray-700 dark:text-gray-300">
            작업 공간 디렉토리 *
          </label>
          <div className="flex items-start gap-4">
            <button
              onClick={handleOpenWorkspace}
              className="px-4 py-2 bg-[#862633] text-white rounded-lg hover:bg-[#a62f3f] 
                dark:bg-[#862633] dark:hover:bg-[#a62f3f] transition-colors shrink-0"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                className="h-5 w-5"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fillRule="evenodd"
                  d="M2 6a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1H2V6zm0 3v6a2 2 0 002 2h12a2 2 0 002-2V9H2z"
                  clipRule="evenodd"
                />
              </svg>
            </button>
            <div className="flex flex-col gap-1 min-w-0 flex-1">
              <div className="break-all">
                <p className="text-m text-gray-600 dark:text-gray-400">
                  {isExpanded ? workspace : truncateWorkspace(workspace)}
                </p>
              </div>
              {workspace.length > 30 && (
                <button
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="text-xs text-[#862633] hover:text-[#a62f3f] underline shrink-0 
                  transition-colors inline-flex items-center gap-1 self-start"
                >
                  {isExpanded ? '접기' : '더보기'}
                  <svg
                    className={`w-4 h-4 transition-transform ${
                      isExpanded ? 'rotate-180' : ''
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

        {/* 필수 입력 필드들 */}
        <div className="space-y-4 p-4 rounded-lg bg-gray-100 dark:bg-gray-800">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              그룹 이름 *
            </label>
            <input
              type="text"
              value={group}
              onChange={(e) => setGroup(e.target.value)}
              className="w-full px-4 py-2 border rounded-lg focus:outline-none
                focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700
                border-gray-300 dark:border-gray-600 dark:text-gray-300"
              placeholder="그룹 이름을 입력하세요"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              닉네임 *
            </label>
            <input
              type="text"
              value={nickname}
              onChange={(e) => setNickname(e.target.value)}
              className="w-full px-4 py-2 border rounded-lg focus:outline-none
                focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700
                border-gray-300 dark:border-gray-600 dark:text-gray-300"
              placeholder="사용할 닉네임을 입력하세요"
              required
            />
          </div>
        </div>

        {/* 선택적 입력 필드들 */}
        <div className="space-y-4 p-4 rounded-lg bg-gray-100 dark:bg-gray-800">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
              추가 설정 (선택)
            </span>
            <button
              onClick={() => setShowOptional(!showOptional)}
              className="flex items-center gap-2 text-sm text-[#862633] hover:text-[#a62f3f] dark:text-gray-400 dark:hover:text-gray-300 underline"
            >
              {showOptional ? '접기' : '더보기'}
              <svg
                className={`w-4 h-4 transition-transform ${
                  showOptional ? 'rotate-180' : ''
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
          </div>

          {showOptional && (
            <div className="space-y-4 mt-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  도메인 (선택)
                </label>
                <input
                  type="text"
                  value={domain}
                  onChange={(e) => setDomain(e.target.value)}
                  className="w-full px-4 py-2 border rounded-lg focus:outline-none
                    focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700
                    border-gray-300 dark:border-gray-600 dark:text-gray-300"
                  placeholder="도메인을 입력하세요"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  해시 (선택)
                </label>
                <input
                  type="text"
                  value={hash}
                  onChange={(e) => setHash(e.target.value)}
                  className="w-full px-4 py-2 border rounded-lg focus:outline-none
                    focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700
                    border-gray-300 dark:border-gray-600 dark:text-gray-300"
                  placeholder="해시를 입력하세요"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    서버 포트 (선택)
                  </label>
                  <input
                    type="number"
                    value={serverPort}
                    onChange={(e) => setServerPort(e.target.value)}
                    className="w-full px-4 py-2 border rounded-lg focus:outline-none
                      focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700
                      border-gray-300 dark:border-gray-600 dark:text-gray-300"
                    placeholder="포트 번호"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    P2P 포트 (선택)
                  </label>
                  <input
                    type="number"
                    value={p2pPort}
                    onChange={(e) => setP2pPort(e.target.value)}
                    className="w-full px-4 py-2 border rounded-lg focus:outline-none
                      focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-700
                      border-gray-300 dark:border-gray-600 dark:text-gray-300"
                    placeholder="포트 번호"
                  />
                </div>
              </div>
            </div>
          )}
        </div>

        {/* 저장 버튼 */}
        <div className="pt-6">
          <button
            onClick={saveSetting}
            className="w-full px-4 py-3 bg-[#862633] text-white rounded-lg
              hover:bg-[#a62f3f] dark:bg-[#862633] dark:hover:bg-[#a62f3f]
              transition-colors font-medium"
          >
            설정 저장
          </button>
          {!isFirst && (
            <button
              className="w-full mt-2 px-4 py-3 bg-gray-400 text-white rounded-lg
                hover:bg-gray-600 dark:bg-gray-600 dark:hover:bg-gray-500
                transition-colors font-medium"
              onClick={goMain}
            >
              취소
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default Settings;
