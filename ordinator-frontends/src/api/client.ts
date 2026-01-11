import type { AuthConfig } from "@/types/dto/AuthConfig";
import type { LocalAuthPayload } from "@/types/dto/LocalAuthPayload";
import type { LoginResponse } from "@/types/dto/LoginResponse";
import type { SchedulingData } from "@/types/dto/SchedulingData";



class ApiClient {
    private baseUrl: string;
    private getAccessToken: () => string | null;
    private onTokenExpired: () => Promise<string>;

    constructor(baseUrl: string, getAccessToken: () => string | null, onTokenExpired: () => Promise<string>) {
        this.baseUrl = baseUrl;
        this.getAccessToken = getAccessToken;
        this.onTokenExpired = onTokenExpired;
    }

    private async request(endpoint: string, options?: RequestInit) {
        const token = this.getAccessToken();
        const headers = {
            'Content-Type': 'application/json',
            ...(token && { Authorization: `Bearer ${token}`}),
            ...options?.headers,
        };

        let response = await fetch(`${this.baseUrl}${endpoint}`, {
            ...options,
            headers
        });

        // Handle UNAUTHORIZED
        if (response.status === 401 && !endpoint.includes("/auth/refresh") && !endpoint.includes("/auth/login")) {
            const newToken = await this.onTokenExpired(); // Call /refresh

            response = await fetch(`${this.baseUrl}${endpoint}`, {
                ...options,
                headers: {
                    'Content-Type': 'application/json',
                    Authorization: `Bearer ${newToken}`,
                    ...options?.headers,
                }
            });
        }

        if (!response.ok) {
            const body = await response.text();
            throw new Error(`(${response.status} ${body})`);
        }

        return response;
    }

    ///////////////////////////////////////////////////////
    // Here gores the Api Calls
    ///////////////////////////////////////////////////////
    async getAuthConfig(): Promise<AuthConfig> {
        const endpoint = `/api/v1/auth/config`;

        const res = await this.request(
            endpoint
        );

        return await res.json() as AuthConfig;
    }

    async tryLocalLogin(
        payload: LocalAuthPayload
    ): Promise<string> {
        const endpoint = "/api/v1/auth/login";
        const options = {
            method: "POST",
            body: JSON.stringify(payload)
        };

        const res = await this.request(endpoint,options);
        if (!res.ok) {
            throw new Error("Invalid credentials");
        } else {
            const data = await res.json() as LoginResponse;

            localStorage.setItem("access_token", data.access_token);
            localStorage.setItem("refresh_token", data.refresh_token);
            return "Login succesful"

        }

    }




    // workorders 
    async fetchWorkOrders(
        asset: string,
        periods?: string[],
    ): Promise<SchedulingData[]> {
        const params = new URLSearchParams();

        if (periods) {
            periods.forEach((p) => params.append("periods", p));
        }

        const endpoint = `/api/v1/scheduler/work_orders_with_scheduling/${asset}?${params.toString()}`;
        const res = await this.request(
            endpoint
        );

        // The Rust DTO is `Vec<SingleRowDto>` => serialises to a bare JSON array
        return await res.json() as SchedulingData[];
    }

}

export const apiClient = new ApiClient(
    import.meta.env.VITE_API_BASE_URL || "",
    () => localStorage.getItem("access_token"),
    async () => {
        const refreshToken = localStorage.getItem("refresh_token");

        if (!refreshToken) {
            throw new Error('No refresh token available');
        }

        const endpoint = `${import.meta.env.VITE_API_BASE_URL}/api/v1/auth/refresh`;
        const response = await fetch(endpoint, {method: "POST", headers: {'Content-Type': 'application/json'}, body: JSON.stringify(refreshToken)});

        if (!response.ok) {
            localStorage.removeItem("access_token");
            localStorage.removeItem("refresh_token");
            throw new Error('Token refresh failed');
        }

        const data = await response.json() as LoginResponse;

        localStorage.setItem("access_token", data.access_token);
        localStorage.setItem("refresh_token", data.refresh_token);

        return data.access_token;
    }
);
