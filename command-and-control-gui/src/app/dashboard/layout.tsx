"use client";
import {Logo} from "@/components/logo";
import {
    ActionIcon,
    AppShell,
    AppShellAside,
    AppShellMain,
    AppShellNavbar,
    AppShellSection,
    Button,
    CloseButton,
    NavLink,
    ScrollArea,
    Tooltip,
} from "@mantine/core";
import {useDisclosure} from "@mantine/hooks";
import {IconChevronLeft, IconLayoutDashboard, IconLogout,} from "@tabler/icons-react";
import {usePathname, useRouter,} from "next/navigation";
import {ReactNode} from "react";

export default function Layout({children}: {
    children: ReactNode
}) {
    const path = usePathname();
    const router = useRouter();
    const [aside_disclosure, toggle_aside_disclosure] = useDisclosure(true);

    return (
        <AppShell
            navbar={{
                width: 300,
                breakpoint: "sm",
            }}
            aside={{
                width: 300,
                breakpoint: "md",
                collapsed: {
                    desktop: !aside_disclosure,
                    mobile: !aside_disclosure,
                },
            }}
            padding={"md"}
            layout={"alt"}
        >
            <AppShellNavbar p="md">
                <AppShellSection>
                    <Logo/>
                </AppShellSection>
                <AppShellSection grow
                                 my="xl"
                                 component={ScrollArea}
                >
                    <NavLink href="/dashboard"
                             active={path === "/dashboard"}
                             label={"Dashboard"}
                             leftSection={<IconLayoutDashboard size={20}/>}
                    />
                </AppShellSection>
                <AppShellSection>
                    <Button onClick={async () => {
                        const {AuthenticationCtx} = await import("@/context/authentication");
                        AuthenticationCtx.logout(router)
                    }}
                            fullWidth
                            color={"dark.9"}
                            rightSection={<IconLogout size={20}/>}
                    >
                        Logout
                    </Button>
                </AppShellSection>
            </AppShellNavbar>
            <AppShellMain id={"main"}>

                {children}
            </AppShellMain>
            <AppShellAside p="md">
                <CloseButton onClick={toggle_aside_disclosure.close}/>
            </AppShellAside>
            {
                !aside_disclosure && (
                    <Tooltip label={"Open sidebar"}
                             color={"dark.9"}
                             position={"left"}
                             withArrow
                             arrowSize={10}
                             arrowRadius={3}
                    >
                        <ActionIcon color={"dark.9"}
                                    size={"lg"}
                                    className="absolute right-0 top-24 translate-x-1/3 transition-all duration-300"
                                    onClick={toggle_aside_disclosure.open}
                        >
                            <IconChevronLeft/>
                        </ActionIcon>
                    </Tooltip>
                )
            }
        </AppShell>
    );
}