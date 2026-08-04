export function BlobBackground() {
  return (
    <div className="fixed inset-0 z-[-1] overflow-hidden pointer-events-none radial-bg">
      <div className="noise-bg" />
      <div className="grid-bg" />
      
      {/* Primary Blob */}
      <div 
        className="absolute top-[-10%] left-1/2 -translate-x-1/2 w-[900px] h-[1400px] rounded-full blur-[150px] opacity-25 animate-float"
        style={{ background: 'var(--color-accent)' }}
      />
      
      {/* Secondary Blob */}
      <div 
        className="absolute top-[20%] left-[-10%] w-[600px] h-[800px] rounded-full blur-[120px] opacity-15 animate-float-slow"
        style={{ background: 'rgb(192, 132, 252)' }}
      />
      
      {/* Tertiary Blob */}
      <div 
        className="absolute bottom-[-10%] right-[-5%] w-[500px] h-[700px] rounded-full blur-[100px] opacity-10 animate-float"
        style={{ background: 'var(--color-accent)' }}
      />
    </div>
  );
}
