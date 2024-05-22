"use client"
import Image from "next/image";
import {Button, Card, PasswordInput, rgba, Stack, TextInput, Title, useMantineTheme} from "@mantine/core";


export default function Home() {
    const theme = useMantineTheme();
    return (
        <main className={"bg-black flex items-center justify-center h-dvh w-dvw"}>
            <div className={"absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2"}>
                <div className="glitch">
                    <Image src={"/images/globe.png"} alt={"globe"} width={600} height={600}/>
                    <div className="glitch__layers">
                        <div className="glitch__layer"></div>
                        <div className="glitch__layer"></div>
                        <div className="glitch__layer"></div>
                    </div>
                </div>
            </div>
            <Card bg={rgba(theme.colors.dark[9], .9)} padding={"xl"}>
                <Stack>
                    <Title order={1} fz={"h2"}>Login to RS2 instance</Title>
                    <TextInput label={"Host"} placeholder={"127.0.0.1:8080"}/>
                    <TextInput label={"Username"} placeholder={"john-doe"}/>
                    <PasswordInput label={"Password"} placeholder={"••••••••"}/>
                    <Button color={"violet"}>Login</Button>
                </Stack>
            </Card>
        </main>
    );
}
