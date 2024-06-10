"use client";
import { AgentsDatatable } from "@/components/agents-datatable";
import { Terminal } from "@/components/terminal";
import { Agent } from "@/interfaces/agent";
import {
    ActionIcon,
    Group,
    Tabs,
    TabsList,
    TabsPanel,
    TabsTab,
    Text,
    ThemeIcon,
    Tooltip,
} from "@mantine/core";
import {
    IconLayoutColumns,
    IconLayoutRows,
    IconTerminal,
} from "@tabler/icons-react";
import { useRouter } from "next/navigation";
import {
    JSX,
    useCallback,
    useEffect,
    useState,
} from "react";
import "./page.css";
import Resizable from "react-resizable-layout";

const sample_data: Agent[] = [
    {
        id:               "aa112233",
        operative_system: "Windows",
        hostname:         "host1",
        domain:           "example.com",
        username:         "user1",
        ip:               "1.1.1.1",
        process_id:       1234,
        parent_process_id: 5678,
        process_name:     "cmd.exe",
        elevated:         false,
        cwd:              "C:\\Users\\user1",
    },
    {
        id:               "bb445566",
        operative_system: "Linux",
        hostname:         "host2",
        domain:           "example.com",
        username:         "user2",
        ip:               "2.2.2.2",
        process_id:       2345,
        parent_process_id: 6789,
        process_name:     "bash",
        elevated:         true,
        cwd:              "/home/user2",
    },
];

export default function Page() {
    // Redirect to the login page if the user is not authenticated
    const router = useRouter();
    useEffect(() => {
        import("@/context/authentication").then(({ AuthenticationCtx }) => {
            if (!AuthenticationCtx.is_authenticated) {
                router.push("/");
            }
        });
    }, [ router ]);

    // control the active tab
    const [ active_tab, set_active_tab ] = useState<string | null>("global");

    // control the terminals
    const [ terminals, set_terminals ] = useState<{
        [x: string]: (position: number) => JSX.Element
    }>({});

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
        import("@/context/authentication").then(({ AuthenticationCtx }) => {
            set_username(AuthenticationCtx.username);

            set_terminals((terminals) => {
                return {
                    ...terminals,
                    global: (position) => (
                        <Terminal hostname={ "RS2" }
                                  username={ AuthenticationCtx.username }
                                  cwd={ "~" }
                                  style={ {
                                      minHeight: `calc(100dvh - ${ position }px - var(--mantine-spacing-xl, 0) * 4)`,
                                      maxHeight: `calc(100dvh - ${ position }px - var(--mantine-spacing-xl, 0) * 4)`,
                                  } }
                                  dropTerminalHandle={ dropTerminalHandle }
                        />
                    ),
                };
            });
        });
    }, []);

    const addTerminalHandle = useCallback((hostname: string, cwd: string) => {
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

    return (
        <Resizable axis={ "y" }
                   min={ 200 }
                   max={ 500 }
                   initial={ 500 }
        >
            {
                ({
                    position,
                    separatorProps,
                }) => (
                    <>
                        <AgentsDatatable agents={ sample_data }
                                         style={ {
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