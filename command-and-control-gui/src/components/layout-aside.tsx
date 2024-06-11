import { Log } from "@/components/log";
import { dayjs } from "@/helpers/dayjs";
import { ILog } from "@/interfaces/log";
import {
    ActionIcon,
    Button,
    Center,
    CloseButton,
    Group,
    Menu,
    MenuDropdown,
    MenuItem,
    MenuLabel,
    MenuTarget,
    ScrollAreaAutosize,
    Stack,
    Tabs,
    TabsPanel,
    Title,
    Tooltip,
} from "@mantine/core";
import {
    notifications,
    Notifications,
    notificationsStore,
} from "@mantine/notifications";
import {
    IconBell,
    IconDotsVertical,
    IconLayoutSidebarRightCollapse,
    IconLayoutSidebarRightExpand,
    IconLogs,
} from "@tabler/icons-react";
import { title } from "radash";
import {
    Dispatch,
    FC,
    JSX,
    SetStateAction,
    useEffect,
    useRef,
    useState,
} from "react";

interface PersistedAsidePanel {
    active_tab: string;
    aside_size: number;
}

interface LayoutAsideProps {
    toggle_aside_disclosure: {
        readonly open: () => void,
        readonly close: () => void,
        readonly toggle: () => void
    };
    set_aside_size: Dispatch<SetStateAction<number>>;
    aside_size: number;
}

const sample_logs = Array.from({ length: 100 }, (_, i) => ({
    timestamp: dayjs().unix(),
    extra:     {
        uid: "test",
    },
    level:     "INFO",
    title:     "Log title",
    message:   "log message".repeat(12),
} as ILog));

export const LayoutAside: FC<LayoutAsideProps> = (
    {
        toggle_aside_disclosure,
        set_aside_size,
        aside_size,
    },
) => {
    const [ active_tab, set_active_tab ] = useState<string | null>(null);
    const ref = useRef<HTMLDivElement | null>(null);
    const [ empty_notification_state, set_empty_notification_state ] = useState<JSX.Element | null>(
        <Center mt={ "xl" }>
            <Stack gap={ "xs" }>
                <Title order={ 6 }
                       className="text-center"
                >
                    No notification found
                </Title>
                <Button onClick={ () => notifications.show({ message: "This is a test notification" }) }>
                    Send a test notification
                </Button>
            </Stack>
        </Center>,
    );

    // subscribe to notifications store to show a message when there are no notifications
    useEffect(() => {
        notificationsStore.subscribe((v) => {
            if (v.notifications.length === 0) {
                set_empty_notification_state(
                    <Center mt={ "xl" }>
                        <Stack gap={ "xs" }>
                            <Title order={ 6 }
                                   className="text-center"
                            >
                                No notification found
                            </Title>
                            <Button onClick={ () => notifications.show({ message: "This is a test notification" }) }>
                                Send a test notification
                            </Button>
                        </Stack>
                    </Center>,
                );
            }
            else {
                set_empty_notification_state(null);
            }
        });
    }, []);

    // persist active tab in local storage to restore it on page reload
    useEffect(
        () => {
            if (active_tab) {
                localStorage.setItem(
                    "aside-panel",
                    JSON.stringify({
                        active_tab,
                        aside_size,
                    } as PersistedAsidePanel),
                );
            }
        },
        [
            active_tab,
            aside_size,
        ],
    );

    // restore active tab from local storage on page load
    useEffect(() => {
        const raw_data = localStorage.getItem("aside-panel");

        if (!raw_data) {
            set_active_tab("notifications");
            return;
        }

        const aside_panel: PersistedAsidePanel = JSON.parse(raw_data);

        // ensure that the tab is valid
        if ([
            "notifications",
            "logs",
        ].includes(aside_panel.active_tab)) {
            set_active_tab(aside_panel.active_tab);
        }
        else {
            set_active_tab("notifications");
        }

        set_aside_size(aside_panel.aside_size);
    }, []);

    return (
        <>
            <Group justify={ "space-between" }>
                <CloseButton onClick={ toggle_aside_disclosure.close } />
                <Title order={ 4 }
                       mx={ "auto" }
                >
                    {
                        title(active_tab)
                    }
                </Title>
                <Menu shadow={ "md" }
                      width={ 250 }
                      withArrow
                      arrowSize={ 10 }
                      arrowRadius={ 3 }
                      closeOnItemClick={ false }
                >
                    <MenuTarget>
                        <Tooltip label={ "Sidebar menu" }
                                 color={ "dark.9" }
                                 position={ "left" }
                                 withArrow
                                 arrowSize={ 10 }
                                 arrowRadius={ 3 }
                        >
                            <ActionIcon variant={ "light" }>
                                <IconDotsVertical size={ 20 } />
                            </ActionIcon>
                        </Tooltip>
                    </MenuTarget>
                    <MenuDropdown>
                        <MenuLabel>
                            Dimensions
                        </MenuLabel>
                        <MenuItem leftSection={ <IconLayoutSidebarRightExpand size={ 16 } /> }
                                  onClick={ () => set_aside_size(old => Math.min(1000, old + 50)) }
                        >
                            Expand
                        </MenuItem>
                        <MenuItem leftSection={ <IconLayoutSidebarRightCollapse size={ 16 } /> }
                                  onClick={ () => set_aside_size(old => Math.max(300, old - 50)) }
                        >
                            Shrink
                        </MenuItem>
                        <MenuLabel>
                            Modes
                        </MenuLabel>
                        <MenuItem leftSection={ <IconBell size={ 16 } /> }
                                  onClick={ () => set_active_tab("notifications") }
                                  bg={ active_tab === "notifications" ? "violet.9" : undefined }
                        >
                            Notifications
                        </MenuItem>
                        <MenuItem leftSection={ <IconLogs size={ 16 } /> }
                                  onClick={ () => set_active_tab("logs") }
                                  bg={ active_tab === "logs" ? "violet.9" : undefined }
                        >
                            Logs
                        </MenuItem>
                    </MenuDropdown>
                </Menu>
            </Group>
            <Tabs value={ active_tab }
                  mt={ "md" }
                  ref={ ref }
                  className="overflow-hidden max-h-full"
            >
                <TabsPanel value={ "notifications" }
                           className={ "max-h-full overflow-x-hidden" }
                           component={ ScrollAreaAutosize }
                    // @ts-ignore
                           offsetScrollbars
                           scrollbars={ "y" }
                >
                    <Notifications withinPortal={ false }
                                   autoClose={ false }
                                   containerWidth={ ref.current?.clientWidth }
                                   limit={ Infinity }
                                   position={ "top-right" }
                                   zIndex={ 0 }
                                   styles={ {
                                       root: {
                                           position: "relative",
                                           inset:    "0",
                                           width:    "unset",
                                       },
                                   } }
                    />
                    { empty_notification_state }
                </TabsPanel>
                <TabsPanel value={ "logs" }
                           className={ "max-h-full overflow-hidden" }
                           component={ ScrollAreaAutosize }
                >
                    <Log logs={ sample_logs } />
                </TabsPanel>
            </Tabs>
        </>
    );
};