import { ResourceCard } from "@scipo-code/shared";
import { useParams } from "react-router-dom";




export default function ResourceView() {
  const { asset } = useParams();
  

  return (
    <div>
      {asset}
      <ResourceCard />
    </div>

  )
}
