import { useState, useEffect } from 'react';
import { useNavigate, Routes, Route } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import Settings from './Setting.tsx';
import { appConfigDir, homeDir } from '@tauri-apps/api/path';
import DeviceExplorer from './explorer/DeviceExplorer';

// 메인 페이지
function MainPage() {
  const [greetMsg, setGreetMsg] = useState('');
  const [workspace, setWorkspace] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const [isWorkspaceExpanded, setIsWorkspaceExpanded] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    async function is_first() {
      try {
        const savedir = await appConfigDir();
        const homedir = await homeDir();
        await invoke('set_config_path', { savedir, homedir });
        const is_first_run = await invoke<boolean>('is_first_run');
        if (is_first_run) {
          navigate('/settings', { replace: true });
          return;
        }
      } catch (error) {
        console.error('Error fetching init:', error);
      }
    }

    async function greet() {
      try {
        const nickname = await invoke<String>('get_nickname');
        setGreetMsg(`${nickname}`);
        const workspace = await invoke<String>('get_workspace');
        setWorkspace(`${workspace}`);
      } catch (error) {
        console.error('Error fetching nickname:', error);
      }
    }

    async function init() {
      try {
        await invoke('load_config');
        await invoke('init_client');
        await greet();
        setIsConnected(true);
      } catch (error) {
        console.error('Error fetching init:', error);
      }
    }

    const runEffects = async () => {
      await is_first();
      init();
    };

    runEffects();
  }, [navigate]);

  const truncateWorkspace = (path: string) => {
    if (!isWorkspaceExpanded && path.length > 30) {
      return path.substring(0, 30) + '...';
    }
    return path;
  };

  return (
    <div className="main dark:bg-gray-900">
      <div className="main-container max-w-[500px] dark:bg-gray-800">
        {/* KU-Drive 타이틀 섹션 */}
        <div className="p-6 rounded-t-lg border-b dark:border-gray-700">
          <div className="flex items-center justify-center gap-4">
            <img
              src="/raw_icon.svg"
              alt="KU Drive Icon"
              className="w-12 h-12 drop-shadow-lg transform hover:scale-110 transition-all duration-300"
            />
            <div className="flex flex-col items-start -space-y-1">
              <h1 className="text-3xl font-medium tracking-tight leading-none mb-0 bg-gradient-to-br from-[#862633] to-[#b33344] bg-clip-text text-transparent">
                KU-Drive
              </h1>
              <span className="text-s font-medium text-gray-500 dark:text-gray-400 tracking-widest uppercase mt-0">
                분산 파일 공유 시스템
              </span>
            </div>
          </div>
        </div>

        <div className="main-content p-5 space-y-5">
          {/* 사용자 프로필 섹션 */}
          <div className="flex items-center gap-3 pb-3 border-b border-gray-200 dark:border-gray-700">
            <div className="bg-[#862633] rounded-full p-2.5">
              <svg
                className="w-6 h-6 text-white"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                />
              </svg>
            </div>
            <div className="text-left">
              <h3 className="text-m font-bold text-gray-800 dark:text-gray-200">
                사용자
              </h3>
              <p className="text-m text-gray-500 dark:text-gray-400">
                {greetMsg}
              </p>
            </div>
          </div>

          {/* 작업공간 정보 섹션 */}
          <div className="flex items-start gap-3 pb-3 border-b border-gray-200 dark:border-gray-700">
            <div className="bg-[#862633] rounded-full p-2.5 shrink-0">
              <svg
                className="w-6 h-6 text-white"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                />
              </svg>
            </div>
            <div className="text-left min-w-0 flex-1">
              <h3 className="text-base font-semibold text-gray-700 dark:text-gray-300">
                작업 공간
              </h3>
              <div className="flex flex-col gap-1">
                <div className="break-all">
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    {isWorkspaceExpanded
                      ? workspace
                      : truncateWorkspace(workspace)}
                  </p>
                </div>
                {workspace.length > 30 && (
                  <button
                    onClick={() => setIsWorkspaceExpanded(!isWorkspaceExpanded)}
                    className="text-xs text-[#862633] hover:text-[#a62f3f] underline shrink-0 transition-colors inline-flex items-center gap-1 self-start"
                  >
                    {isWorkspaceExpanded ? '접기' : '더보기'}
                    <svg
                      className={`w-4 h-4 transition-transform ${
                        isWorkspaceExpanded ? 'rotate-180' : ''
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

          {/* 연결 ��태 섹션 */}
          <div className="flex items-center gap-3">
            <div className="bg-[#862633] rounded-full p-2.5">
              <svg
                className="w-6 h-6 text-white"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  d="M5 12h14M12 5l7 7-7 7"
                />
              </svg>
            </div>
            <div className="text-left">
              <h3 className="text-base font-semibold text-gray-700 dark:text-gray-300">
                연결 상태
              </h3>
              <div className="flex items-center gap-2">
                <div
                  className={`w-2 h-2 rounded-full ${
                    isConnected ? 'bg-[#00ad00]' : 'bg-yellow-500'
                  }`}
                ></div>
                <div className="flex items-center gap-2">
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    {isConnected ? '서버에 연결됨' : '서버에 연결 중'}
                  </p>
                  {!isConnected && (
                    <svg
                      className="animate-spin h-4 w-4 text-yellow-500"
                      xmlns="http://www.w3.org/2000/svg"
                      fill="none"
                      viewBox="0 0 24 24"
                    >
                      <circle
                        className="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        strokeWidth="4"
                      ></circle>
                      <path
                        className="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      ></path>
                    </svg>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* 기존 버튼 그룹 */}
        <div className="button-group flex flex-col gap-4 my-4">
          {isConnected && (
            <button
              onClick={() => navigate('/device-explorer')}
              className="button green flex items-center gap-3 justify-center w-80 py-4 dark:bg-opacity-90 dark:hover:bg-opacity-100"
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
                  strokeWidth="2"
                  d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z"
                />
              </svg>
              파일 탐색기
            </button>
          )}
          <button
            onClick={() => navigate('/settings', { replace: true })}
            className="button bg-gray-400 dark:bg-gray-600 text-white flex items-center gap-3 justify-center w-80 py-4 dark:hover:bg-gray-700"
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
            설정
          </button>
        </div>
      </div>
    </div>
  );
}

// App 컴포넌트
function App() {
  return (
    <Routes>
      <Route path="/" element={<MainPage />} />
      <Route path="/settings" element={<Settings />} />
      <Route path="/device-explorer" element={<DeviceExplorer />} />
    </Routes>
  );
}

export default App;
