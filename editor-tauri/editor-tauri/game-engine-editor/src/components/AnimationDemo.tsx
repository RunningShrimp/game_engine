import React from 'react';

/**
 * AnimationDemo Component
 *
 * This component showcases all available animations in the system.
 * Use it as a reference for implementing animations throughout the app.
 */

export const AnimationDemo: React.FC = () => {
  return (
    <div className="p-8 space-y-8 bg-slate-900 text-slate-200 min-h-screen">
      <h1 className="text-3xl font-bold animate-fade-in">Animation System Demo</h1>

      {/* Fade Animations */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-blue-400">Fade Animations</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="animate-fade-in bg-slate-800 p-4 rounded hover-lift">
            Fade In
          </div>
          <div className="animate-fade-in-up bg-slate-800 p-4 rounded hover-lift">
            Fade In Up
          </div>
          <div className="animate-fade-in-down bg-slate-800 p-4 rounded hover-lift">
            Fade In Down
          </div>
          <div className="animate-fade-in delay-300 bg-slate-800 p-4 rounded hover-lift">
            Delayed Fade
          </div>
        </div>
      </section>

      {/* Slide Animations */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-green-400">Slide Animations</h2>
        <div className="grid grid-cols-2 gap-4">
          <div className="animate-slide-in-left bg-slate-800 p-4 rounded">
            Slide from Left
          </div>
          <div className="animate-slide-in-right bg-slate-800 p-4 rounded">
            Slide from Right
          </div>
          <div className="animate-slide-in-top bg-slate-800 p-4 rounded">
            Slide from Top
          </div>
          <div className="animate-slide-in-bottom bg-slate-800 p-4 rounded">
            Slide from Bottom
          </div>
        </div>
      </section>

      {/* Scale Animations */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-purple-400">Scale Animations</h2>
        <div className="grid grid-cols-3 gap-4">
          <div className="animate-scale-in bg-slate-800 p-4 rounded hover-lift">
            Scale In
          </div>
          <div className="animate-scale-in-bounce bg-slate-800 p-4 rounded hover-lift">
            Scale Bounce
          </div>
          <div className="animate-scale-in duration-700 bg-slate-800 p-4 rounded hover-lift">
            Slow Scale
          </div>
        </div>
      </section>

      {/* Loading States */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-yellow-400">Loading States</h2>
        <div className="space-y-3">
          <div className="animate-shimmer h-12 w-full rounded bg-gradient-to-r from-slate-800 to-slate-700"></div>
          <div className="skeleton h-8 w-3/4 rounded"></div>
          <div className="skeleton-dark h-8 w-1/2 rounded"></div>
          <div className="flex items-center gap-4">
            <div className="animate-spin-fast w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full"></div>
            <div className="animate-spin-medium w-8 h-8 border-4 border-green-500 border-t-transparent rounded-full"></div>
            <div className="animate-spin-slow w-8 h-8 border-4 border-purple-500 border-t-transparent rounded-full"></div>
          </div>
        </div>
      </section>

      {/* Spin Animations */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-red-400">Spin Animations</h2>
        <div className="flex gap-8 items-center">
          <div className="text-center">
            <div className="animate-spin-fast text-4xl mb-2">🔄</div>
            <p className="text-sm">Fast</p>
          </div>
          <div className="text-center">
            <div className="animate-spin-medium text-4xl mb-2">🔄</div>
            <p className="text-sm">Medium</p>
          </div>
          <div className="text-center">
            <div className="animate-spin-slow text-4xl mb-2">🔄</div>
            <p className="text-sm">Slow</p>
          </div>
        </div>
      </section>

      {/* Bounce Animations */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-pink-400">Bounce Animations</h2>
        <div className="flex gap-4">
          <div className="animate-bounce-custom bg-slate-800 p-4 rounded">
            Continuous Bounce
          </div>
          <div className="animate-bounce-in bg-slate-800 p-4 rounded">
            Bounce In
          </div>
        </div>
      </section>

      {/* Pulse Animations */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-cyan-400">Pulse Animations</h2>
        <div className="flex gap-4 items-center">
          <div className="animate-pulse-custom bg-green-500/20 text-green-400 px-4 py-2 rounded">
            ● Live
          </div>
          <div className="animate-pulse-custom bg-blue-500/20 text-blue-400 px-4 py-2 rounded">
            ● Recording
          </div>
          <div className="animate-pulse-custom bg-red-500/20 text-red-400 px-4 py-2 rounded">
            ● Alert
          </div>
        </div>
      </section>

      {/* Hover Effects */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-orange-400">Hover Effects</h2>
        <div className="grid grid-cols-3 gap-4">
          <button className="hover-lift bg-slate-800 p-4 rounded transition-smooth">
            Hover Lift
          </button>
          <button className="hover-scale bg-slate-800 p-4 rounded transition-smooth">
            Hover Scale
          </button>
          <button className="hover-glow bg-slate-800 p-4 rounded transition-smooth">
            Hover Glow
          </button>
        </div>
      </section>

      {/* Transition Speeds */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-indigo-400">Transition Speeds</h2>
        <div className="grid grid-cols-3 gap-4">
          <button className="transition-fast hover:bg-slate-700 bg-slate-800 p-4 rounded">
            Fast (150ms)
          </button>
          <button className="transition-smooth hover:bg-slate-700 bg-slate-800 p-4 rounded">
            Smooth (300ms)
          </button>
          <button className="transition-slow hover:bg-slate-700 bg-slate-800 p-4 rounded">
            Slow (500ms)
          </button>
        </div>
      </section>

      {/* Staggered List */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-teal-400">Staggered List</h2>
        <ul className="stagger-in space-y-2">
          {[1, 2, 3, 4, 5].map((item) => (
            <li key={item} className="bg-slate-800 p-3 rounded hover-lift transition-smooth">
              Item {item} - appears with staggered delay
            </li>
          ))}
        </ul>
      </section>

      {/* Modal Examples */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-rose-400">Modal Animation</h2>
        <div className="relative">
          <div className="modal-enter animate-scale-in bg-slate-800 p-6 rounded-lg max-w-md">
            <h3 className="text-lg font-semibold mb-2">Modal Content</h3>
            <p className="text-slate-400 mb-4">
              This modal uses the scale-in animation with a smooth enter effect.
            </p>
            <button className="bg-blue-500 hover:bg-blue-600 px-4 py-2 rounded transition-smooth">
              Close
            </button>
          </div>
        </div>
      </section>

      {/* Panel Animation */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-emerald-400">Panel Animation</h2>
        <div className="panel-slide-in-bottom animate-slide-in-bottom bg-slate-800 p-4 rounded">
          This panel slides in from the bottom, perfect for timelines or notifications.
        </div>
      </section>

      {/* Status Indicators */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-violet-400">Status Indicators</h2>
        <div className="flex gap-4 items-center">
          <span className="text-green-400 animate-pulse-custom">● Playing</span>
          <span className="text-yellow-400">● Paused</span>
          <span className="text-slate-400">○ Stopped</span>
        </div>
      </section>

      {/* GPU Accelerated Examples */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-amber-400">Performance Optimized</h2>
        <div className="grid grid-cols-2 gap-4">
          <div className="gpu-accelerated will-change-transform animate-scale-in bg-slate-800 p-4 rounded hover-lift">
            GPU Accelerated
          </div>
          <div className="will-change-opacity animate-fade-in bg-slate-800 p-4 rounded hover-lift">
            Opacity Optimized
          </div>
        </div>
      </section>

      {/* Combined Animations */}
      <section className="space-y-4">
        <h2 className="text-xl font-semibold text-lime-400">Combined Effects</h2>
        <div className="animate-fade-in-up animate-scale-in duration-500 bg-gradient-to-r from-blue-500 to-purple-500 p-6 rounded-lg hover-lift hover-scale transition-smooth">
          <h3 className="text-xl font-bold mb-2">Multiple Animations</h3>
          <p>
            This element combines fade-in-up, scale-in, hover-lift, and hover-scale for a
            rich interactive experience.
          </p>
        </div>
      </section>
    </div>
  );
};

export default AnimationDemo;
