#[macro_export]
macro_rules! simple_impl_opt {
    ($ty:ident, $name:expr, [$($style:pat),+], $check:expr, $deactivate:expr) => {
        impl AOpt for $ty {
            fn _get_uid(&self) -> Uid {
                self.uid
            }

            fn _set_uid(&mut self, uid: Uid) {
                self.uid = uid;
            }

            fn _get_setted(&self) -> bool {
                self.setted
            }

            fn _set_setted(&mut self, setted: bool) {
                self.setted = setted;
            }

            fn _get_type_name(&self) -> Str {
                $name
            }

            fn _is_deactivate_style(&self) -> bool {
                ($deactivate)(self)
            }

            fn _match_style(&self, style: OptStyle) -> bool {
                match style {
                    $(
                        $style => true,
                    )+
                    _ => false
                }
            }

            fn _get_name(&self) -> Str {
                self.name.clone()
            }

            fn _set_name(&mut self, name: Str) {
                self.name = name;
            }

            fn _get_prefix(&self) -> Option<Str> {
                self.prefix.clone()
            }

            fn _set_prefix(&mut self, prefix: Option<Str>) {
                self.prefix = prefix;
            }

            fn _get_optional(&self) -> bool {
                self.optional
            }

            fn _set_optional(&mut self, optional: bool) {
                self.optional = optional;
            }

            fn _get_alias(&self) -> Option<&Vec<(Str, Str)>> {
                Some(&self.alias)
            }

            fn _add_alias(&mut self, prefix: Str, name: Str) {
                self.alias.push((prefix, name))
            }

            fn _rem_alias(&mut self, prefix: Str, name: Str) {
                if let Some((i, _)) = self
                    .alias
                    .iter()
                    .enumerate()
                    .find(|(_, v)| v.0 == prefix && v.1 == name)
                {
                    self.alias.remove(i);
                }
            }

            fn _match_alias(&self, prefix: Str, name: Str) -> bool {
                self.alias.iter().any(|v| v.0 == prefix && v.1 == name)
            }

            fn _get_hint(&self) -> Str {
                self.help.get_hint()
            }

            fn _get_help(&self) -> Str {
                self.help.get_help()
            }

            fn _set_hint(&mut self, hint: Str) {
                self.help.set_hint(hint);
            }

            fn _set_help(&mut self, help: Str) {
                self.help.set_help(help);
            }

            fn _get_index(&self) -> Option<&OptIndex> {
                None
            }

            fn _set_index(&mut self, _index: Option<OptIndex>) {}

            fn _match_index(&self, index: Option<(usize, usize)>) -> bool {
                index.is_none()
            }

            fn _chk_value(
                &mut self,
                _arg: Option<Str>,
                disable: bool,
                _index: (usize, usize),
            ) -> Result<bool, Error> {
                ($check)(self, _arg, disable, _index)
            }

            fn _has_callback(&self) -> bool {
                self.callback.is_some()
            }

            fn _invoke(&mut self, ser: &mut Services, ctx: Context) -> Result<Option<Str>, Error> {
                let store_func = std::mem::replace(&mut self.callback, None);

                if let Some(mut func) = store_func {
                    let ret = func.invoke(self, ser, ctx);

                    // store the callback back
                    self.callback = Some(func);
                    ret
                } else {
                    Ok(None)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! simple_impl_noa {
    ($ty:ident, $name:expr, [$($style:pat),+], $name_mat:expr, $optional_get:expr, $optional_set:expr, $index_set:expr, $index_mat:expr) => {
        impl AOpt for $ty {
            fn _get_uid(&self) -> Uid {
                self.uid
            }

            fn _set_uid(&mut self, uid: Uid) {
                self.uid = uid;
            }

            fn _get_setted(&self) -> bool {
                self.setted
            }

            fn _set_setted(&mut self, setted: bool) {
                self.setted = setted;
            }

            fn _get_type_name(&self) -> Str {
                $name
            }

            fn _is_deactivate_style(&self) -> bool {
                false
            }

            fn _match_style(&self, style: OptStyle) -> bool {
                match style {
                    $(
                        $style => true,
                    )+
                    _ => false
                }
            }

            fn _get_name(&self) -> Str {
                self.name.clone()
            }

            fn _set_name(&mut self, name: Str) {
                self.name = name;
            }

            fn _match_name(&self, name: Str) -> bool {
                ($name_mat)(self, name)
            }

            fn _get_prefix(&self) -> Option<Str> {
                None
            }

            fn _set_prefix(&mut self, _prefix: Option<Str>) { }


            fn _get_optional(&self) -> bool {
                ($optional_get)(self)
            }

            fn _set_optional(&mut self, optional: bool) {
                ($optional_set)(self, optional)
            }

            fn _get_alias(&self) -> Option<&Vec<(Str, Str)>> {
                None
            }

            fn _add_alias(&mut self, _prefix: Str, _name: Str) { }

            fn _rem_alias(&mut self, _prefix: Str, _name: Str) { }

            fn _match_alias(&self, _prefix: Str, _name: Str) -> bool {
                false
            }

            fn _get_hint(&self) -> Str {
                self.help.get_hint()
            }

            fn _get_help(&self) -> Str {
                self.help.get_help()
            }

            fn _set_hint(&mut self, hint: Str) {
                self.help.set_hint(hint);
            }

            fn _set_help(&mut self, help: Str) {
                self.help.set_help(help);
            }

            fn _get_index(&self) -> Option<&OptIndex> {
                self.index.as_ref()
            }

            fn _set_index(&mut self, index: Option<OptIndex>) {
                ($index_set)(self, index)
            }

            fn _match_index(&self, index: Option<(usize, usize)>) -> bool {
                ($index_mat)(self, index)
            }

            fn _chk_value(
                &mut self,
                _arg: Option<Str>,
                _disable: bool,
                _index: (usize, usize),
            ) -> Result<bool, Error> {
                Ok(true)
            }

            fn _has_callback(&self) -> bool {
                self.callback.is_some()
            }

            fn _invoke(&mut self, ser: &mut Services, ctx: Context) -> Result<Option<Str>, Error> {
                let store_func = std::mem::replace(&mut self.callback, None);

                if let Some(mut func) = store_func {
                    let ret = func.invoke(self, ser, ctx);

                    // store the callback back
                    self.callback = Some(func);
                    ret
                } else {
                    Ok(None)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! simple_impl_opt_for {
    ($type:ty) => {
        impl Opt for $type {
            fn reset(&mut self) {
                self._reset()
            }

            fn check(&self) -> bool {
                self._check()
            }

            fn get_uid(&self) -> Uid {
                self._get_uid()
            }

            fn set_uid(&mut self, uid: Uid) {
                self._set_uid(uid)
            }

            fn set_setted(&mut self, setted: bool) {
                self._set_setted(setted)
            }

            fn get_setted(&self) -> bool {
                self._get_setted()
            }

            fn get_type_name(&self) -> Str {
                self._get_type_name()
            }

            fn is_deactivate_style(&self) -> bool {
                self._is_deactivate_style()
            }

            fn match_style(&self, style: OptStyle) -> bool {
                self._match_style(style)
            }

            fn has_callback(&self) -> bool {
                self._has_callback()
            }

            fn invoke_callback(
                &mut self,
                ser: &mut Services,
                ctx: Context,
            ) -> Result<Option<Str>, Error> {
                self._invoke(ser, ctx)
            }

            fn check_value(
                &mut self,
                arg: Option<Str>,
                disable: bool,
                index: (usize, usize),
            ) -> Result<bool, Error> {
                self._chk_value(arg, disable, index)
            }
        }

        impl Name for $type {
            fn get_name(&self) -> Str {
                self._get_name()
            }

            fn set_name(&mut self, name: Str) {
                self._set_name(name)
            }

            fn match_name(&self, name: Str) -> bool {
                self._match_name(name)
            }
        }

        impl Prefix for $type {
            fn get_prefix(&self) -> Option<Str> {
                self._get_prefix()
            }

            fn set_prefix(&mut self, prefix: Option<Str>) {
                self._set_prefix(prefix)
            }

            fn match_prefix(&self, prefix: Option<Str>) -> bool {
                self._match_prefix(prefix)
            }
        }

        impl Optional for $type {
            fn get_optional(&self) -> bool {
                self._get_optional()
            }

            fn set_optional(&mut self, optional: bool) {
                self._set_optional(optional)
            }

            fn match_optional(&self, optional: bool) -> bool {
                self._match_optional(optional)
            }
        }

        impl Alias for $type {
            fn get_alias(&self) -> Option<&Vec<(Str, Str)>> {
                self._get_alias()
            }

            fn add_alias(&mut self, prefix: Str, name: Str) {
                self._add_alias(prefix, name)
            }

            fn rem_alias(&mut self, prefix: Str, name: Str) {
                self._rem_alias(prefix, name)
            }

            fn match_alias(&self, prefix: Str, name: Str) -> bool {
                self._match_alias(prefix, name)
            }
        }

        impl Help for $type {
            fn get_hint(&self) -> Str {
                self._get_hint()
            }

            fn get_help(&self) -> Str {
                self._get_help()
            }

            fn set_hint(&mut self, hint: Str) {
                self._set_hint(hint)
            }

            fn set_help(&mut self, help: Str) {
                self._set_help(help)
            }
        }

        impl Index for $type {
            fn get_index(&self) -> Option<&OptIndex> {
                self._get_index()
            }

            fn set_index(&mut self, index: Option<OptIndex>) {
                self._set_index(index)
            }

            fn match_index(&self, index: Option<(usize, usize)>) -> bool {
                self._match_index(index)
            }
        }
    };
}

#[macro_export]
macro_rules! simple_impl_creator_for {
    ($type:ty) => {
        impl Creator for $type {
            type Opt = <$type as ACreator>::Opt;

            type Config = <$type as ACreator>::Config;

            type Error = Error;

            fn get_type_name(&self) -> Str {
                self._get_type_name()
            }

            fn is_support_deactivate_style(&self) -> bool {
                self._support_deactivate_style()
            }

            fn create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
                self._create_with(config)
            }
        }
    };
}
