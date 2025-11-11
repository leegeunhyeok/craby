'use client';

import { useSearchContext, useSidebar } from 'fumadocs-ui/provider';

export function Navbar() {
  const { open, setOpen } = useSidebar();
  const { setOpenSearch } = useSearchContext();

  return (
    <div className="bg-fd-background border-b border-fd-border w-full h-[64px] fixed top-0 z-10">
      <button type="button" onClick={() => setOpen(!open)}>
        Toggle Sidebar
      </button>
      <button type="button" onClick={() => setOpenSearch(true)}>
        Toggle Search
      </button>
    </div>
  );
}
