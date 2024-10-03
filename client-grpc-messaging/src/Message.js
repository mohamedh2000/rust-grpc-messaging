export const Message = ({userName, message}) => {
  return (
    <div class="flex items-start gap-2.5">
      <img
        class="w-8 h-8 rounded-full"
        src="/docs/images/people/profile-picture-3.jpg"
        alt="Jese image"
      />
      <div class="flex flex-col w-full max-w-[320px] leading-1.5 p-4 border-gray-200 bg-gray-100 rounded-e-xl rounded-es-xl dark:bg-gray-700">
        <div class="flex">
          <span class="flex text-sm font-semibold mr-auto text-gray-900 dark:text-white">
            {userName}
          </span>
          <span class="flex text-sm font-normal text-gray-500 dark:text-gray-400 right-0">
            11:46
          </span>
        </div>
        <p class="text-sm font-normal py-2.5 text-left text-gray-900 dark:text-white">
          {message}
        </p>
      </div>
    </div>
  );
};

export default Message;
