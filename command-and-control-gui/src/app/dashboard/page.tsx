"use client";
/*import { AuthenticationCtx } from "@/context/authentication";
 import { useRouter } from "next/navigation";*/

import SplitPane from "@/components/split-pane/split-pane";
import {
    ActionIcon,
    Menu,
    MenuDivider,
    MenuDropdown,
    MenuItem,
    MenuLabel,
    MenuTarget,
    Text,
    Tooltip,
} from "@mantine/core";
import {
    IconBrandApple,
    IconBrandDebian,
    IconBrandWindows,
    IconBug,
    IconCheck,
    IconChevronUp,
    IconDotsVertical,
    IconSelector,
    IconSkull,
    IconTableColumn,
    IconTerminal,
    IconX,
} from "@tabler/icons-react";
import {DataTable, DataTableSortStatus, useDataTableColumns,} from "mantine-datatable";
import {alphabetical} from "radash";
import {useEffect, useState} from "react";
import "./page.css";
import "@/components/split-pane/style.css";
import {Terminal} from "@/components/terminal";

interface Agent {
    /**
     * The unique identifier for the agent (cuid2)
     */
    id: string;
    /**
     * The OS name
     */
    operative_system: string;
    /**
     * The victim hostname
     */
    hostname: string;
    /**
     * The domain of the victim
     */
    domain: string;
    /**
     * The username of whose runs the agent
     */
    username: string;
    /**
     * The internal IP of the victim
     */
    ip: string;
    /**
     * The process ID of the agent
     */
    process_id: number;
    /**
     * The parent process ID of the agent
     */
    parent_process_id: number;
    /**
     * The process name of the agent
     */
    process_name: string;
    /**
     * Whether the agent is running as elevated
     */
    elevated: boolean;
}

const sample_data: Agent[] = [
    {
        id: "aa112233",
        operative_system: "Windows",
        hostname: "host1",
        domain: "example.com",
        username: "user1",
        ip: "1.1.1.1",
        process_id: 1234,
        parent_process_id: 5678,
        process_name: "cmd.exe",
        elevated: false,
    },
    {
        id: "bb445566",
        operative_system: "Linux",
        hostname: "host2",
        domain: "example.com",
        username: "user2",
        ip: "2.2.2.2",
        process_id: 2345,
        parent_process_id: 6789,
        process_name: "bash",
        elevated: true,
    },
];

const column_toggle_key = "agents-toggleable";

export default function Page() {
    /*const router = useRouter();
     if (!AuthenticationCtx.is_authenticated) {
     router.push("/");
     }*/

    const [sortStatus, setSortStatus] = useState<DataTableSortStatus<Agent>>({
        columnAccessor: "id",
        direction: "asc",
    });

    const [selectedRecords, setSelectedRecords] = useState<Agent[]>([]);

    const [records, setRecords] = useState(alphabetical(
        sample_data,
        v => v[sortStatus.columnAccessor as keyof Agent].toString(),
        sortStatus.direction === "asc" ? "asc" : "desc",
    ));

    useEffect(() => {
        const data = alphabetical(
            sample_data,
            v => v[sortStatus.columnAccessor as keyof Agent].toString(),
            sortStatus.direction === "asc" ? "asc" : "desc",
        );
        setRecords(data);
    }, [sortStatus]);

    const {
        effectiveColumns,
        resetColumnsToggle,
        setColumnsToggle,
    } = useDataTableColumns({
        key: column_toggle_key,
        columns: [
            {
                accessor: "id",
                title: "ID",
                sortable: true,
                toggleable: true,
                defaultToggle: false,
            },
            {
                accessor: "operative_system",
                sortable: true,
                toggleable: true,
                // this column has a custom title
                title: "OS",
                // right-align column
                textAlign: "center",
                render: ({operative_system}) => {
                    if (operative_system.toLowerCase() === "windows") {
                        return <IconBrandWindows size={24}/>;
                    } else if (operative_system.toLowerCase() === "linux") {
                        return <IconBrandDebian size={24}/>;
                    } else if (operative_system.toLowerCase() === "macos") {
                        return <IconBrandApple size={24}/>;
                    } else {
                        return <Text size={"sm"}>{operative_system}</Text>;
                    }
                },
            },
            {
                accessor: "hostname",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "domain",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "username",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "ip",
                sortable: true,
                toggleable: true,
                title: "IP Address",
            },
            {
                accessor: "process_id",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "parent_process_id",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "process_name",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "elevated",
                textAlign: "center",
                sortable: true,
                toggleable: true,
                render: ({elevated}) => {
                    return elevated ? <IconCheck size={20}/> : <IconX size={20}/>;
                },
            },
            {
                accessor: "actions",
                title: (
                    <Menu shadow={"md"}
                          width={250}
                          withArrow
                          arrowSize={10}
                          arrowRadius={3}
                    >
                        <MenuTarget>
                            <Tooltip label={"Bulk actions"}
                                     color={"dark.9"}
                                     position={"left"}
                                     withArrow
                                     arrowSize={10}
                                     arrowRadius={3}
                            >
                                <ActionIcon variant={"light"}>
                                    <IconDotsVertical size={20}/>
                                </ActionIcon>
                            </Tooltip>
                        </MenuTarget>
                        <MenuDropdown>
                            <MenuLabel>
                                Table
                            </MenuLabel>
                            <MenuItem onClick={() => resetColumnsToggle()}
                                      leftSection={<IconTableColumn size={14}/>}
                            >
                                Reset visible columns
                            </MenuItem>
                            <MenuDivider/>
                            <MenuLabel>
                                Bulk actions
                            </MenuLabel>
                            <MenuItem onClick={() => console.log("Bulk terminal")}
                                      leftSection={<IconTerminal size={14}/>}
                                      disabled={selectedRecords.length === 0}
                            >
                                Terminal
                            </MenuItem>
                            <MenuItem onClick={() => console.log("Bulk terminate")}
                                      leftSection={<IconSkull size={14}/>}
                                      color={"red"}
                                      disabled={selectedRecords.length === 0}
                            >
                                Terminate
                            </MenuItem>
                        </MenuDropdown>
                    </Menu>
                ),
                render: ({id}) => (
                    <Menu shadow={"md"}
                          width={250}
                          withArrow
                          arrowSize={10}
                          arrowRadius={3}
                    >
                        <MenuTarget>
                            <ActionIcon variant={"light"}>
                                <IconDotsVertical size={20}/>
                            </ActionIcon>
                        </MenuTarget>
                        <MenuDropdown>
                            <MenuLabel>
                                Actions
                            </MenuLabel>
                            <MenuItem onClick={() => console.log(`Terminal: ${id}`)}
                                      leftSection={<IconTerminal size={14}/>}
                            >
                                Terminal
                            </MenuItem>
                            <MenuItem onClick={() => console.log(`Terminate: ${id}`)}
                                      leftSection={<IconSkull size={14}/>}
                                      color={"red"}
                            >
                                Terminate
                            </MenuItem>
                        </MenuDropdown>
                    </Menu>
                ),
            },
        ],
    });

    return (
        <SplitPane split={"horizontal"}
                   minSize={400}
                   defaultSize={600}
        >
            <DataTable
                className="w-full"
                mx={"xl"}
                my={"md"}
                withRowBorders
                withColumnBorders
                horizontalSpacing={"xs"}
                verticalSpacing={"sm"}
                fz={"sm"}
                verticalAlign={"center"}
                highlightOnHover
                minHeight={200}
                maxHeight={600}
                noRecordsText={"No agents found"}
                noRecordsIcon={<IconBug size={30}
                                        className="mb-2"
                />}
                sortStatus={sortStatus}
                onSortStatusChange={setSortStatus}
                sortIcons={{
                    sorted: <IconChevronUp size={14}/>,
                    unsorted: <IconSelector size={14}/>,
                }}
                selectedRecords={selectedRecords}
                onSelectedRecordsChange={setSelectedRecords}
                selectionTrigger={"cell"}
                records={records}
                // @ts-ignore
                columns={effectiveColumns}
                storeColumnsKey={column_toggle_key}
            />
            <Terminal hostname={"host1"} username={"ebalo"} cwd={"/home/ebalo"}/>
        </SplitPane>
    );
}