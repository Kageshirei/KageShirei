"use client";

import { AuthenticationCtx } from "@/context/authentication";
import {
    Button,
    Card,
    PasswordInput,
    rgba,
    Stack,
    TextInput,
    Title,
    useMantineTheme,
} from "@mantine/core";
import { useForm } from "@mantine/form";
import Image from "next/image";
import { useRouter } from "next/navigation";
import {
    useCallback,
    useEffect,
} from "react";

const authenticate = async (values: {
    host: string,
    username: string,
    password: string
}) => {
    const response = await fetch(`http://${ values.host }/authenticate`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body:   JSON.stringify({
            username: values.username,
            password: values.password,
        }),
    });

    return await response.json();
};

export default function Home() {
    const router = useRouter();

    useEffect(() => {
        if (AuthenticationCtx.is_authenticated) {
            router.push("/dashboard");
        }
    }, [ router ]);

    const theme = useMantineTheme();

    const form = useForm({
        initialValues: {
            host: "",
            username: "",
            password: "",
        },
    });

    const handleAuthentication = useCallback(
        async (values: typeof form["values"]) => {
            const response = await authenticate(values);

            if (response.error) {
                form.setErrors({
                    username: response.error,
                    password: response.error,
                });
            }
            else {
                AuthenticationCtx.authenticate({
                    host:       values.host,
                    bearer:     response.token,
                    username:   values.username,
                    expires_in: response.expires_in,
                });
                router.push("/dashboard");
            }
        },
        [
            form,
            router,
        ],
    );

    return (
        <main className={ "bg-black flex items-center justify-center h-dvh w-dvw" }>
            <div className={ "absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2" }>
                <div className="glitch">
                    <Image src={ "/images/globe.png" }
                           alt={ "globe" }
                           width={ 600 }
                           height={ 600 }
                    />
                    <div className="glitch__layers">
                        <div className="glitch__layer"></div>
                        <div className="glitch__layer"></div>
                        <div className="glitch__layer"></div>
                    </div>
                </div>
            </div>
            <form onSubmit={ form.onSubmit(handleAuthentication) }>
                <Card bg={ rgba(theme.colors.dark[9], .9) }
                      padding={ "xl" }
                >
                    <Stack>
                        <Title order={ 1 }
                               fz={ "h2" }
                        >
                            Login to KageShirei instance
                        </Title>
                        <TextInput label={ "Host" }
                                   placeholder={ "127.0.0.1:8080" }
                                   { ...form.getInputProps("host") }
                        />
                        <TextInput label={ "Username" }
                                   placeholder={ "john-doe" }
                                   { ...form.getInputProps("username") }
                        />
                        <PasswordInput label={ "Password" }
                                       placeholder={ "••••••••" }
                                       { ...form.getInputProps("password") }
                        />
                        <Button type={ "submit" }
                        >
                            Login
                        </Button>
                    </Stack>
                </Card>
            </form>
        </main>
    );
}
