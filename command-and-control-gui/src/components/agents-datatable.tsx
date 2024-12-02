import {GlobalSessions} from "@/context/globals/sessions";
import {Agent, NetworkInterfaceArray} from "@/interfaces/agent";
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
    IconBrandWindows,
    IconBug,
    IconChevronUp,
    IconDotsVertical,
    IconSelector,
    IconSkull,
    IconTableColumn,
    IconTerminal,
} from "@tabler/icons-react";
import {DataTable, DataTableSortStatus, useDataTableColumns,} from "mantine-datatable";
import {alphabetical} from "radash";
import {CSSProperties, FC, useEffect, useState,} from "react";
import {useSnapshot} from "valtio";

interface AgentsDatatableProps {
    addTerminalHandle: (hostname: string, cwd: string, id: string) => void;
    style?: CSSProperties;
}

const column_toggle_key = "agents-toggleable";

export const AgentsDatatable: FC<AgentsDatatableProps> = ({
                                                              style,
                                                              addTerminalHandle,
                                                          }) => {
    const sessions = useSnapshot(GlobalSessions);

    const [sortStatus, setSortStatus] = useState<DataTableSortStatus<Agent>>({
        columnAccessor: "id",
        direction: "asc",
    });

    const [selectedRecords, setSelectedRecords] = useState<Agent[]>([]);

    const [records, setRecords] = useState(alphabetical(
        sessions.data,
        v => v[sortStatus.columnAccessor as keyof Agent].toString(),
        sortStatus.direction === "asc" ? "asc" : "desc",
    ));

    useEffect(
        () => {
            const data = alphabetical(
                sessions.data,
                v => v[sortStatus.columnAccessor as keyof Agent].toString(),
                sortStatus.direction === "asc" ? "asc" : "desc",
            );

            console.log(data)
            setRecords(data);
        },
        [
            sessions,
            sortStatus,
        ],
    );

    const {
        effectiveColumns,
        resetColumnsToggle,
    } = useDataTableColumns<Agent>({
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
                accessor: "operating_system",
                sortable: true,
                toggleable: true,
                // this column has a custom title
                title: "OS",
                // right-align column
                textAlign: "center",
                render: ({operating_system}) => {
                    if (!operating_system) {
                        return <Text size={"sm"}>Unknown</Text>;
                    }

                    if (operating_system.toLowerCase() === "windows") {
                        return <IconBrandWindows size={24}/>;
                    } else if (operating_system.toLowerCase() === "linux") {
                        return "*nix";
                    } else if (operating_system.toLowerCase() === "macos") {
                        return <IconBrandApple size={24}/>;
                    } else {
                        return <Text size={"sm"}>{operating_system}</Text>;
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
                accessor: "network_interfaces",
                sortable: true,
                toggleable: true,
                title: "Network Interfaces",
                render: ({network_interfaces}) => {
                    return (<>
                        {(network_interfaces as NetworkInterfaceArray).network_interfaces.map((network_interface, index) => (
                            <Text size={"sm"} key={network_interface.address || index}
                                  className={"clear-both my-2"}>{network_interface.address}</Text>
                        ))}
                    </>)
                }
            },
            {
                accessor: "pid",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "ppid",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "process_name",
                sortable: true,
                toggleable: true,
            },
            {
                accessor: "cwd",
                title: "Current Working Directory",
                sortable: true,
                toggleable: true,
                defaultToggle: false,
            },
            {
                accessor: "integrity",
                sortable: true,
                toggleable: true,
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
                            {/*<MenuItem onClick={ () => console.log("Bulk terminal") }
                                                leftSection={ <IconTerminal size={ 14 } /> }
                                                disabled={ selectedRecords.length === 0 }
                                      >
                                          Terminal
                                       </MenuItem>*/}
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
                render: ({
                             id,
                             cwd,
                             hostname,
                         }) => (
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
                            <MenuItem onClick={() => addTerminalHandle(hostname, cwd, id)}
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
        <DataTable
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
            fetching={sessions.is_fetching}
            // @ts-ignore
            columns={effectiveColumns}
            storeColumnsKey={column_toggle_key}
            style={style}
        />
    );
};