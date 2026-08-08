function FixtureButton(_props: { className: string; variant: string; children: string }) {
    return null;
}

export function PascalFixture() {
    return (
        <>
            <FixtureButton className="border shadow-lg bg-gradient-to-r" variant="outline">Invalid</FixtureButton>
            <FixtureButton className={`border ${String(1)}`} variant={"outline"}>Also invalid</FixtureButton>
        </>
    );
}
