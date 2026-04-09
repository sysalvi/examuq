<?php

namespace App\Filament\Resources\ExamSessions\Pages;

use App\Filament\Resources\ExamSessions\ExamSessionResource;
use Filament\Resources\Pages\ListRecords;

class ListExamSessions extends ListRecords
{
    protected static string $resource = ExamSessionResource::class;
}
