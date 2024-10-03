import SideBarButton from "./SideBarButton";
import { useState, useEffect } from "react";

const SideBar = () => {



  return (
    <div class="relative flex h-full w-full max-w-[15rem] flex-col rounded-xl bg-white bg-clip-border p-4 text-gray-700 shadow-xl shadow-blue-gray-900/5">
      <nav class="flex min-w-[240px]  flex-col gap-1 p-2 font-sans text-base font-normal text-blue-gray-700">





        <SideBarButton text={"Friend1"}/>
        <SideBarButton text={"Log Out"}/>

      </nav>
    </div>
  );
}

export default SideBar;



