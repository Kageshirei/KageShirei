import { AuthenticationCtx } from "@/context/authentication";
import { useRouter } from "next/navigation";
import { useEffect } from "react";

/**
 * Ensure that the user is authenticated before rendering the component.
 * Redirects to the login page if the user is not authenticated.
 *
 * NOTE: It's possible to see a flash of the component before the redirect happens.
 */
export function useEnsureUserIsAuthenticated(): void {
    const router = useRouter();
    useEffect(() => {
        // Redirect to the login page if the user is not authenticated
        if (!AuthenticationCtx.is_authenticated) {
            router.push("/");
        }
    }, [ router ]);
}