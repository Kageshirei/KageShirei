"use client";
import { AgentsDatatable } from "@/components/agents-datatable";
import { Terminal } from "@/components/terminal";
import { AuthenticationCtx } from "@/context/authentication";
import {
    getFromLocalStorage,
    persistInLocalStorage,
} from "@/helpers/local-storage";
import { useEnsureUserIsAuthenticated } from "@/hooks/use-ensure-user-is-authenticated";
import {
    Tabs,
    TabsList,
    TabsPanel,
    TabsTab,
    Text,
    ThemeIcon,
} from "@mantine/core";
import { IconTerminal } from "@tabler/icons-react";
import {
    JSX,
    useCallback,
    useEffect,
    useState,
} from "react";
import "./page.css";
import Resizable, { ResizeCallbackArgs } from "react-resizable-layout";

const split_storage_key = "agents-terminal-split";

export default function Page() {
    useEnsureUserIsAuthenticated();

    // control the active tab
    const [ active_tab, set_active_tab ] = useState<string | null>("global");

    // control the terminals
    const [ terminals, set_terminals ] = useState<{
        [x: string]: (position: number) => JSX.Element
    }>({});

    // Drop a terminal from the list of terminals
    const dropTerminalHandle = useCallback((hostname: string) => {
        set_terminals((terminals) => {
            // If the active tab is the one being closed, set the active tab to the previous tab
            const keys = Object.keys(terminals);
            const index = keys.indexOf(hostname);
            const active_index = keys.indexOf(active_tab ?? "");
            const new_active_tab = keys[active_index === index ? Math.max(index - 1, 0) : active_index];
            set_active_tab(new_active_tab);

            const new_terminals = { ...terminals };
            delete new_terminals[hostname];
            return new_terminals;
        });
    }, [ active_tab ]);

    // Get the username from the authentication context
    const [ username, set_username ] = useState("");
    useEffect(() => {
        set_username(AuthenticationCtx.username);
    }, []);

    // Add a terminal to the list of terminals
    const addTerminalHandle = useCallback(
        (hostname: string, cwd: string, id: string) => {
            set_terminals((terminals) => {
                set_active_tab(hostname);
                return {
                    ...terminals,
                    [hostname]: (position) => (
                        <Terminal hostname={ hostname }
                                  username={ username }
                                  cwd={ cwd }
                                  style={ {
                                      minHeight: `calc(100dvh - ${ position }px - var(--mantine-spacing-xl, 0) * 4)`,
                                      maxHeight: `calc(100dvh - ${ position }px - var(--mantine-spacing-xl, 0) * 4)`,
                                  } }
                                  dropTerminalHandle={ dropTerminalHandle }
                                  addTerminalHandle={addTerminalHandle}
                                  session_id={ id }
                        />
                    ),
                };
            });
        },
        [
            dropTerminalHandle,
            username,
        ],
    );

    // add the global terminal
    useEffect(() => {
        if (!("global" in terminals) && username.length > 0) {
            set_terminals((terminals) => {
                if ("global" in terminals) {
                    return terminals;
                }

                return {
                    ...terminals,
                    global: (position) => (
                        <Terminal hostname={"RS2"}
                                  username={username}
                                  cwd={`/home/${username}`}
                                  style={{
                                      minHeight: `calc(100dvh - ${position}px - var(--mantine-spacing-xl, 0) * 4)`,
                                      maxHeight: `calc(100dvh - ${position}px - var(--mantine-spacing-xl, 0) * 4)`,
                                  }}
                                  dropTerminalHandle={dropTerminalHandle}
                                  addTerminalHandle={addTerminalHandle}
                                  session_id={"global"}
                        />
                    ),
                };
            });
        }
    }, [addTerminalHandle, dropTerminalHandle, username]);

    return (
        <Resizable axis={ "y" }
                   min={ 200 }
                   max={ 500 }
                   initial={ getFromLocalStorage<ResizeCallbackArgs>(split_storage_key)?.position ?? 500 }
                   onResizeEnd={ (args) => {
                       persistInLocalStorage(split_storage_key, args);
                   } }
        >
            {
                ({
                    position,
                    separatorProps,
                }) => (
                    <>
                        <AgentsDatatable style={ {
                                             height: position,
                                         } }
                                         addTerminalHandle={ addTerminalHandle }
                        />
                        <div
                            className="cursor-row-resize h-0.5 w-full bg-transparent border-solid border-0 border-t border-t-zinc-600 py-1"
                            { ...separatorProps } />
                        <Tabs variant={ "outline" }
                              style={ {
                                  minHeight: `calc(100dvh - ${ position }px - var(--mantine-spacing-md, 0) * 5)`,
                              } }
                              className={ "relative" }
                              value={ active_tab }
                              onChange={ set_active_tab }
                        >
                            {/*
                             TODO: Implement split terminal functionality
                             <Group className={ "absolute right-0 top-0 translate-y-1/2" }
                             gap={ "xs" }
                             >
                             <Tooltip label={ "Split terminal vertically" }
                             withArrow
                             arrowSize={ 10 }
                             arrowRadius={ 3 }
                             color={ "dark.9" }
                             >
                             <ActionIcon variant={ "light" }
                             onClick={ () => console.log("Split terminal vertically") }
                             size={ "sm" }
                             >
                             <IconLayoutColumns size={ 16 } />
                             </ActionIcon>
                             </Tooltip>
                             <Tooltip label={ "Split terminal horizontally" }
                             withArrow
                             arrowSize={ 10 }
                             arrowRadius={ 3 }
                             color={ "dark.9" }
                             >
                             <ActionIcon variant={ "light" }
                             onClick={ () => console.log("Split terminal horizontally") }
                             size={ "sm" }
                             >
                             <IconLayoutRows size={ 16 } />
                             </ActionIcon>
                             </Tooltip>
                             </Group>
                             <TabsList className="flex-nowrap overflow-x-auto w-[94%]">
                             */ }
                            <TabsList className="flex-nowrap overflow-x-auto">
                                {
                                    Object.keys(terminals).map((tab) => (
                                        <TabsTab key={ tab }
                                                 value={ tab }
                                        >
                                            {
                                                tab === "global"
                                                ? (
                                                    <ThemeIcon variant={ "filled" }
                                                               size={ "sm" }
                                                    >
                                                        <IconTerminal size={ 12 } />
                                                    </ThemeIcon>
                                                )
                                                : (
                                                    <Text size={ "xs" }>
                                                        @{ tab }
                                                    </Text>
                                                )
                                            }
                                        </TabsTab>
                                    ))
                                }
                            </TabsList>
                            {
                                Object.entries(terminals).map(([ tab, terminal ]) => (
                                    <TabsPanel key={ tab }
                                               value={ tab }
                                    >
                                        {
                                            terminal(position)
                                        }
                                    </TabsPanel>
                                ))
                            }
                        </Tabs>
                    </>
                )
            }
        </Resizable>
    );
}