import { apiClient } from "@/api/client";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import type { LocalAuthPayload } from "@/types/dto/LocalAuthPayload";
import { useState } from "react";
import { useNavigate } from "react-router-dom";

export default function LoginPage() {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const navigate = useNavigate();
  // const [missingCredentials, setMissingCredentials] = useState(false);

  const submitCredentials = async (username: string, passwd: string) => {
    const loginCredentials = {
      client_id: username,
      client_secret: passwd,
    } as LocalAuthPayload;

    try {
      let login_res = await apiClient.tryLocalLogin(loginCredentials);
      console.log(login_res.assets);
      setError("");
      navigate(`/${login_res.role.toLocaleLowerCase()}/${login_res.assets[0].toLocaleLowerCase()}`)
    } catch (error) {
      if (error instanceof Error) {
        setError(error.message);
      } else {
        setError("An unknown error occurred");
      }
    }

    
  }

  return (
    <div className="flex min-h-svh w-full items-center justify-center p-6 md:p-10">
      <div className="w-full max-w-sm">
        <Card>
          <Input placeholder="Username" onChange={(e) => setUsername(e.target.value)} value={username} />
          <Input type="password" placeholder="Password" onChange={(e) => setPassword(e.target.value)} value={password} />
          <Button variant="default" onClick={() => submitCredentials(username, password)}>Login</Button>
          {error ? <p>Login failed: {error}</p> : ""}
        </Card>
      </div>
    </div>
  
  )
}






