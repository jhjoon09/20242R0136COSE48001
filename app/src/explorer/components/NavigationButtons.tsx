import { VscChevronUp } from 'react-icons/vsc';

interface NavigationButtonsProps {
  onWorkspaceClick: () => void;
  onUpClick: () => void;
  isUpDisabled: boolean;
  showUploadButton?: boolean;
}

const NavigationButtons: React.FC<NavigationButtonsProps> = ({
  onWorkspaceClick,
  onUpClick,
  isUpDisabled,
  showUploadButton = false,
}) => {
  return (
    <div
      className={`fixed right-8 flex flex-col gap-2 transition-all duration-300 ease-in-out text-white
        ${showUploadButton ? 'bottom-24' : 'bottom-8'}`}
    >
      <button
        onClick={onUpClick}
        disabled={isUpDisabled}
        className={`p-3 rounded-full shadow-lg transition-all duration-300
          ${
            isUpDisabled
              ? 'bg-gray-300 cursor-not-allowed dark:bg-gray-700'
              : 'bg-[#862633] hover:bg-[#a62f3f] dark:bg-[#862633] dark:hover:bg-[#a62f3f]'
          }
          transform hover:scale-105`}
        title="상위 디렉토리"
      >
        <VscChevronUp className="w-6 h-6 " />
      </button>
      <button
        onClick={onWorkspaceClick}
        className="p-3 rounded-full bg-[#862633] hover:bg-[#a62f3f] 
          dark:bg-[#862633] dark:hover:bg-[#a62f3f] shadow-lg
          transition-all duration-300 transform hover:scale-105"
        title="작업 공간으로 이동"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          className="w-6 h-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
          />
        </svg>
      </button>
    </div>
  );
};

export default NavigationButtons;
