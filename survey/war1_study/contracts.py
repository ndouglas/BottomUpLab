"""Immutable validated identities shared by W1-A preparation tooling."""

from dataclasses import dataclass

Json = dict[str, object]


@dataclass(frozen=True)
class Key:
    ordinal: int
    arm_id: str
    replicate_index: int


@dataclass(frozen=True)
class Case:
    key: Key
    seed: int


@dataclass(frozen=True)
class Manifest:
    raw: Json
    raw_bytes: bytes
    sha256: str
    purpose: str
    arms: tuple[Json, ...]
    cases: tuple[Case, ...]


class _FrozenDict(dict):
    def _refuse(self, *args, **kwargs):
        raise TypeError("validated manifest is immutable")

    __setitem__ = _refuse
    __delitem__ = _refuse
    clear = _refuse
    pop = _refuse
    popitem = _refuse
    setdefault = _refuse
    update = _refuse
    __ior__ = _refuse


class _FrozenList(list):
    def _refuse(self, *args, **kwargs):
        raise TypeError("validated manifest is immutable")

    __setitem__ = _refuse
    __delitem__ = _refuse
    append = _refuse
    clear = _refuse
    extend = _refuse
    insert = _refuse
    pop = _refuse
    remove = _refuse
    reverse = _refuse
    sort = _refuse
    __iadd__ = _refuse
    __imul__ = _refuse


def _freeze(value):
    if isinstance(value, dict):
        return _FrozenDict((key, _freeze(child)) for key, child in value.items())
    if isinstance(value, list):
        return _FrozenList(_freeze(child) for child in value)
    return value
