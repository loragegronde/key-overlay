import { useEffect } from "react";
import { useOverlayStore } from "../store/useOverlayStore";
import { loadLibrary, saveLibrary } from "../store/persistence";
import { LIBRARY_SCHEMA_VERSION } from "../types";

const SAVE_DEBOUNCE_MS = 400;

/**
 * Loads the layout library on mount and persists it whenever the active
 * profile or the library list changes.
 */
export function useProfilePersistence() {
  useEffect(() => {
    let cancelled = false;

    void loadLibrary().then(({ library, source }) => {
      if (cancelled) return;
      if (source === "migrated") {
        console.info("Migrated saved layout(s) into the layout library");
      }
      useOverlayStore.getState().setLibrary(library);
      useOverlayStore.getState().setHydrated(true);
    });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | undefined;

    const unsubscribe = useOverlayStore.subscribe((state, previous) => {
      if (!state.hydrated) return;
      if (
        state.profile === previous.profile &&
        state.library === previous.library
      ) {
        return;
      }

      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        const { profile, library } = useOverlayStore.getState();
        const profiles = library.profiles.some((p) => p.id === profile.id)
          ? library.profiles.map((p) => (p.id === profile.id ? profile : p))
          : [...library.profiles, profile];

        void saveLibrary({
          version: LIBRARY_SCHEMA_VERSION,
          activeId: profile.id,
          profiles,
        }).catch((error) => {
          console.error("could not save layouts", error);
        });
      }, SAVE_DEBOUNCE_MS);
    });

    return () => {
      if (timer) clearTimeout(timer);
      unsubscribe();
    };
  }, []);
}
