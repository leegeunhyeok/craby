import Image from "next/image";

export function Title() {
  return (
    <div className="flex flex-row items-center justify-center gap-2">
      <Image src="/logo.svg" alt="Craby" width={36} height={36} />
      <p className="text-xl">Craby</p>
    </div>
  )
}
