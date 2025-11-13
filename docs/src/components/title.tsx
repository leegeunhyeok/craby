import Image from 'next/image';
import Link from 'next/link';

const LOGO_SIZE = 40;

export function Title() {
  return (
    <Link href="/" className="flex flex-row items-center justify-center gap-2 mr-5">
      <Image src="/logo.svg" alt="Craby" width={LOGO_SIZE} height={LOGO_SIZE} />
      <p className="text-md font-medium">Craby</p>
    </Link>
  );
}
