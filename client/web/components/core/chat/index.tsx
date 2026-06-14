"use client";

import { useState } from "react";
import { useIsMobile } from "@/hooks/use-mobile";
import { SidebarContainer } from "./sidebar/sidebar-container";
import { ChatareaContainer } from "./chatarea/chatarea-container";
import { AgentsContainer } from "./agents/agents-container";
import { Sheet, SheetContent } from "@/components/ui/sheet";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";


export function ChatContainer() {
  const isMobile = useIsMobile();
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [isRightPanelOpen, setIsRightPanelOpen] = useState(true);

  if (isMobile) {
    return (
      <div className="fixed inset-0 h-screen w-full flex flex-col bg-background overflow-hidden">
        {/* Main Chat Area */}
        <ChatareaContainer onToggleSidebar={() => setIsSidebarOpen(true)} />

        {/* Sliding Left Sidebar Drawer on Mobile */}
        <Sheet open={isSidebarOpen} onOpenChange={setIsSidebarOpen}>
          <SheetContent
            side="left"
            className="w-[280px] p-0 border-r h-full"
            showCloseButton={false}
          >
            <div className="h-full w-full" onClick={() => setIsSidebarOpen(false)}>
              <SidebarContainer />
            </div>
          </SheetContent>
        </Sheet>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 h-screen w-full bg-background overflow-hidden">
      <ResizablePanelGroup
        orientation="horizontal"
        key={isRightPanelOpen ? "with-right" : "no-right"}
      >
        {/* Left Sidebar Panel */}
        <ResizablePanel
          id="sidebar-panel"
          defaultSize={isRightPanelOpen ? "18%" : "20%"}
          minSize="15%"
          maxSize="30%"
        >
          <SidebarContainer />
        </ResizablePanel>

        <ResizableHandle />

        {/* Center Chat Panel */}
        <ResizablePanel
          id="chat-panel"
          defaultSize={isRightPanelOpen ? "58%" : "80%"}
          minSize="30%"
        >
          <ChatareaContainer
            onToggleRightPanel={() => setIsRightPanelOpen(!isRightPanelOpen)}
          />
        </ResizablePanel>

        {/* Right Agent Panel */}
        {isRightPanelOpen && (
          <>
            <ResizableHandle />
            <ResizablePanel
              id="agent-panel"
              defaultSize="24%"
              minSize="20%"
              maxSize="40%"
            >
              <AgentsContainer />
            </ResizablePanel>
          </>
        )}
      </ResizablePanelGroup>
    </div>
  );
}
