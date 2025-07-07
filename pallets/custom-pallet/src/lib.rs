#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame::pallet]
pub mod pallet {
    use super::*;
    use frame::prelude::*;
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    
    pub trait WeightInfo {
    fn set_counter_value() -> Weight;
    fn increment() -> Weight;
    fn decrement() -> Weight;
}

    // Trait de configuração do pallet
    #[pallet::config]
    pub trait Config: frame_system::Config {
        // Define o tipo de evento do pallet
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // Define o valor máximo que o contador pode armazenar
        #[pallet::constant]
        type CounterMaxValue: Get<u32>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// O valor do contador foi definido por Root
        CounterValueSet {
            /// O novo valor definido
            counter_value: u32,
        },
        /// Um usuário incrementou o contador com sucesso
        CounterIncremented {
            /// O novo valor do contador
            counter_value: u32,
            /// A conta que incrementou o contador
            who: T::AccountId,
            /// O valor do incremento
            incremented_amount: u32,
        },
        /// Um usuário decrementou o contador com sucesso
        CounterDecremented {
            /// O novo valor do contador
            counter_value: u32,
            /// A conta que decrementou o contador
            who: T::AccountId,
            /// O valor do decremento
            decremented_amount: u32,
        },
    }

    /// Armazenamento do valor atual do contador
    #[pallet::storage]
    pub type CounterValue<T> = StorageValue<_, u32>;

    /// Mapeamento para rastrear interações por usuário
    #[pallet::storage]
    pub type UserInteractions<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u32>;

    #[pallet::error]
    pub enum Error<T> {
        /// O valor do contador excede o máximo permitido
        CounterValueExceedsMax,
        /// O valor do contador não pode ser menor que zero
        CounterValueBelowZero,
        /// Ocorreu overflow no contador
        CounterOverflow,
        /// Ocorreu overflow nas interações do usuário
        UserInteractionOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Define o valor do contador
        ///
        /// A origem da chamada deve ser _Root_
        ///
        /// - `new_value`: Novo valor para o contador
        ///
        /// Emite o evento `CounterValueSet` quando bem-sucedido
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::set_counter_value())]
            pub fn set_counter_value(
                origin: OriginFor<T>,
                new_value: u32,
            ) -> DispatchResult {
                ensure_root(origin)?;

            ensure!(
                new_value <= T::CounterMaxValue::get(),
                Error::<T>::CounterValueExceedsMax
            );

            CounterValue::<T>::put(new_value);

            Self::deposit_event(Event::<T>::CounterValueSet {
                counter_value: new_value,
            });

            Ok(())
        }

        /// Incrementa o contador por um valor específico
        ///
        /// Pode ser chamado por qualquer conta assinada
        ///
        /// - `amount_to_increment`: Valor a ser incrementado
        ///
        /// Emite o evento `CounterIncremented` quando bem-sucedido
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::increment())]
        pub fn increment(
            origin: OriginFor<T>,
            amount_to_increment: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_value = CounterValue::<T>::get().unwrap_or(0);
            let new_value = current_value
                .checked_add(amount_to_increment)
                .ok_or(Error::<T>::CounterOverflow)?;

            ensure!(
                new_value <= T::CounterMaxValue::get(),
                Error::<T>::CounterValueExceedsMax
            );

            CounterValue::<T>::put(new_value);

            UserInteractions::<T>::try_mutate(&who, |interactions| -> Result<_, Error<T>> {
                let new_interactions = interactions
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or(Error::<T>::UserInteractionOverflow)?;
                *interactions = Some(new_interactions);
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::CounterIncremented {
                counter_value: new_value,
                who,
                incremented_amount: amount_to_increment,
            });

            Ok(())
        }

        /// Decrementa o contador por um valor específico
        ///
        /// Pode ser chamado por qualquer conta assinada
        ///
        /// - `amount_to_decrement`: Valor a ser decrementado
        ///
        /// Emite o evento `CounterDecremented` quando bem-sucedido
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::decrement())]
        pub fn decrement(
            origin: OriginFor<T>,
            amount_to_decrement: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_value = CounterValue::<T>::get().unwrap_or(0);
            let new_value = current_value
                .checked_sub(amount_to_decrement)
                .ok_or(Error::<T>::CounterValueBelowZero)?;

            CounterValue::<T>::put(new_value);

            UserInteractions::<T>::try_mutate(&who, |interactions| -> Result<_, Error<T>> {
                let new_interactions = interactions
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or(Error::<T>::UserInteractionOverflow)?;
                *interactions = Some(new_interactions);
                Ok(())
            })?;

            Self::deposit_event(Event::<T>::CounterDecremented {
                counter_value: new_value,
                who,
                decremented_amount: amount_to_decrement,
            });

            Ok(())
        }
    }
}